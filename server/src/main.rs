use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use cctmog_protocol::*;
use futures::{SinkExt, StreamExt};
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::mpsc;
use uuid::Uuid;

mod game;
mod messages;
// mod persistence;
#[cfg(test)]
mod tests;

use game::*;
use messages::MessageStore;

// ==== knobs ====
const AUTO_START_WHEN_ALL_READY: bool = true; // start as soon as all ready?
const DEALER_MUST_START: bool = false; // only dealer can press "Start hand"
const MAX_PLAYERS: usize = 7; // maximum players per table

#[derive(Clone)]
struct AppState {
    inner: Arc<Mutex<Rooms>>,
    message_store: Arc<MessageStore>,
    distributed_tables: Arc<Mutex<HashMap<String, cctmog_protocol::TableInfo>>>,
}
type Rooms = HashMap<String, game::Room>;

#[tokio::main]
async fn main() {
    // Initialize message store
    let message_store = Arc::new(MessageStore::new("./message_data").unwrap());

    let state = AppState {
        inner: Arc::new(Mutex::new(HashMap::new())),
        message_store,
        distributed_tables: Arc::new(Mutex::new(HashMap::new())),
    };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state.clone());

    let addr = "0.0.0.0:9001";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("server listening on ws://{addr}/ws");
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    let (tx_out, mut rx_out) = tokio::sync::mpsc::unbounded_channel::<ServerToClient>();

    tokio::spawn(async move {
        while let Some(msg) = rx_out.recv().await {
            let text = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(text)).await.is_err() {
                break;
            }
        }
    });

    let my_id = uuid::Uuid::new_v4();
    let _ = tx_out.send(ServerToClient::Hello { your_id: my_id });

    let mut joined_room: Option<String> = None;

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(t) => {
                if let Ok(cmd) = serde_json::from_str::<ClientToServer>(&t) {
                    route_cmd(cmd, &state, &mut joined_room, my_id, &tx_out).await;
                } else {
                    let _ = tx_out.send(ServerToClient::Error {
                        message: "bad json".into(),
                    });
                }
            }
            Message::Close(_) => {
                if let Some(room) = &joined_room {
                    remove_player(&state, room, my_id);
                    remove_spectator(&state, room, my_id);
                }
                break;
            }
            _ => {}
        }
    }
}

async fn route_cmd(
    cmd: ClientToServer,
    state: &AppState,
    joined_room: &mut Option<String>,
    my_id: Uuid,
    tx_out: &mpsc::UnboundedSender<ServerToClient>,
) {
    // --- DEBUG PRINT ---
    eprintln!("[WS] from {} → {:?}", &my_id.to_string()[..8], cmd);

    match cmd {
        ClientToServer::TakeCard => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    player_take_card(r, my_id);
                });
            }
        }
        ClientToServer::Stand => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    player_stand(r, my_id);
                });
            }
        }

        ClientToServer::Join { room, name } => {
            let mut rooms = state.inner.lock();
            let r = rooms.entry(room.clone()).or_insert_with(|| game::Room::new(room.clone()));

            // Check if table is at maximum capacity - if so, join as spectator
            if r.players.len() >= MAX_PLAYERS {
                eprintln!("[SPECTATOR_AUTO] {} auto-joining as spectator (table full)", name);

                // Add as spectator
                r.spectators.push(game::Spectator {
                    id: my_id,
                    name: name.clone(),
                    tx: tx_out.clone(),
                });
                *joined_room = Some(room.clone());

                // Send spectator joined message
                let _ = tx_out.send(ServerToClient::SpectatorJoined {
                    snapshot: game::public_room(r),
                });

                // Notify players that a spectator joined
                for p in r.players.iter() {
                    let _ = p.tx.send(ServerToClient::Info {
                        message: format!("{} joined as spectator (table full)", name),
                    });
                }

                return;
            }

            let seat = r.players.len();
            r.players.push(PlayerSeat {
                id: my_id,
                name,
                chips: 1000,
                folded: false,
                standing: false,
                up_cards: vec![],
                down_cards: vec![],
                ready: false,
                committed_round: 0,
                tx: tx_out.clone(),
            });
            *joined_room = Some(room.clone());
            log_room("JOIN", r);
            broadcast_state(r);
            send_state_to(r, my_id);

            let _ = tx_out.send(ServerToClient::Joined {
                snapshot: game::public_room(r),
                your_seat: seat,
                your_hand: PrivateHand { down_cards: vec![] },
            });
        }
        ClientToServer::Leave => {
            if let Some(room) = joined_room {
                remove_player(state, room, my_id);
            }
        }
        ClientToServer::SitReady => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    if let Some(p) = r.players.iter_mut().find(|p| p.id == my_id) {
                        p.ready = true;
                        eprintln!(
                            "[READY] room={} seat={} now ready; all_ready={}",
                            r.name,
                            r.players
                                .iter()
                                .position(|x| x.id == my_id)
                                .unwrap_or(usize::MAX),
                            r.players.iter().all(|pp| pp.ready)
                        );
                    }
                    log_room("READY", r);
                    broadcast_state(r);
                    send_state_to(r, my_id);

                    if AUTO_START_WHEN_ALL_READY
                        && r.phase == Phase::Lobby
                        && r.players.len() >= 2
                        && r.players.iter().all(|p| p.ready)
                    {
                        eprintln!(
                            "[AUTO-START] room={} players={} all_ready=true phase={:?}",
                            r.name,
                            r.players.len(),
                            r.phase
                        );
                        start_hand(r);
                    }
                });
            }
        }
        ClientToServer::StartHand => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    let starter_seat = match seat_of(r, my_id) {
                        Some(s) => s,
                        None => {
                            eprintln!("[START] rejected: not seated");
                            send_err_to(r, my_id, "You are not seated.");
                            return;
                        }
                    };

                    eprintln!(
                        "[START] attempt: phase={:?} players={} dealer={} starter={}",
                        r.phase,
                        r.players.len(),
                        r.dealer_seat,
                        starter_seat
                    );

                    if r.phase != Phase::Lobby {
                        eprintln!("[START] rejected: phase={:?}", r.phase);
                        send_err_to(r, my_id, format!("Cannot start: phase is {:?}.", r.phase));
                        return;
                    }
                    if r.players.len() < 2 {
                        eprintln!("[START] rejected: players={}", r.players.len());
                        send_err_to(r, my_id, "Need at least 2 players to start.");
                        return;
                    }
                    if let Some(not_ready) = r.players.iter().position(|p| !p.ready) {
                        eprintln!("[START] rejected: seat {} not ready", not_ready);
                        send_err_to(
                            r,
                            my_id,
                            format!("All players must be ready. Seat {} is not.", not_ready),
                        );
                        return;
                    }
                    if DEALER_MUST_START && starter_seat != r.dealer_seat {
                        eprintln!(
                            "[START] rejected: starter={} dealer={} (dealer must start)",
                            starter_seat, r.dealer_seat
                        );
                        send_err_to(
                            r,
                            my_id,
                            format!("Only dealer (seat {}) can start the hand.", r.dealer_seat),
                        );
                        return;
                    }

                    eprintln!("[START] OK → dealing…");
                    start_hand(r);
                    send_state_to(r, my_id);
                });
            }
        }
        ClientToServer::Fold => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    player_fold(r, my_id);
                });
            }
        }
        ClientToServer::Check => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    player_check(r, my_id);
                });
            }
        }
        ClientToServer::Bet => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    player_bet_or_raise(r, my_id, false);
                });
            }
        }
        ClientToServer::Call => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    player_call(r, my_id);
                });
            }
        }
        ClientToServer::Raise => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    player_bet_or_raise(r, my_id, true);
                });
            }
        }
        ClientToServer::Chat { message, scope } => {
            handle_chat_message(state.clone(), my_id, joined_room.clone(), message, scope).await;
        }
        ClientToServer::PrivateMessage { recipient, message } => {
            handle_private_message(state.clone(), my_id, recipient, message).await;
        }
        ClientToServer::ListTables => {
            handle_list_tables(state.clone(), tx_out).await;
        }
        ClientToServer::ScheduleGame { start_time } => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    // Verify player is in the room
                    if game::seat_of(r, my_id).is_none() {
                        send_err_to(r, my_id, "You must be in the room to schedule a game.");
                        return;
                    }

                    // Set the scheduled start time
                    r.scheduled_start = Some(start_time.clone());
                    r.checked_in_players.clear(); // Reset check-ins for new schedule

                    // Notify all players about the scheduled game
                    let info_msg = format!("Game scheduled to start at {}", start_time);
                    for p in r.players.iter() {
                        let _ = p.tx.send(ServerToClient::Info {
                            message: info_msg.clone(),
                        });
                    }

                    eprintln!("[SCHEDULE] Room {} scheduled for {}", r.name, start_time);
                    broadcast_state(r);
                });
            } else {
                let _ = tx_out.send(ServerToClient::Error {
                    message: "You must join a room before scheduling a game.".to_string(),
                });
            }
        }
        ClientToServer::CheckIn => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    // Verify player is in the room
                    if game::seat_of(r, my_id).is_none() {
                        send_err_to(r, my_id, "You must be in the room to check in.");
                        return;
                    }

                    // Check if there's a scheduled game
                    if r.scheduled_start.is_none() {
                        send_err_to(r, my_id, "No game is currently scheduled.");
                        return;
                    }

                    // Add player to checked-in list if not already checked in
                    if !r.checked_in_players.contains(&my_id) {
                        r.checked_in_players.push(my_id);

                        // Find player name for messaging
                        let player_name = r.players.iter()
                            .find(|p| p.id == my_id)
                            .map(|p| p.name.clone())
                            .unwrap_or_else(|| "Unknown".to_string());

                        // Notify all players about the check-in
                        let info_msg = format!("{} has checked in ({}/{})",
                            player_name,
                            r.checked_in_players.len(),
                            r.players.len()
                        );
                        for p in r.players.iter() {
                            let _ = p.tx.send(ServerToClient::Info {
                                message: info_msg.clone(),
                            });
                        }

                        eprintln!("[CHECKIN] {} checked in for room {} ({}/{})",
                            player_name, r.name, r.checked_in_players.len(), r.players.len());
                    } else {
                        send_err_to(r, my_id, "You have already checked in.");
                        return;
                    }

                    broadcast_state(r);
                });
            } else {
                let _ = tx_out.send(ServerToClient::Error {
                    message: "You must join a room before checking in.".to_string(),
                });
            }
        }
        ClientToServer::SelectGameVariant { variant } => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    // Verify player is in the room
                    if game::seat_of(r, my_id).is_none() {
                        send_err_to(r, my_id, "You must be in the room to select game variant.");
                        return;
                    }

                    // Only allow variant selection in lobby phase
                    if r.phase != Phase::Lobby {
                        send_err_to(r, my_id, "Game variant can only be changed in the lobby.");
                        return;
                    }

                    // Update the game variant
                    r.game_variant = variant;

                    // Notify all players about the variant change
                    let info_msg = format!("Game variant changed to {}", variant);
                    for p in r.players.iter() {
                        let _ = p.tx.send(ServerToClient::Info {
                            message: info_msg.clone(),
                        });
                    }

                    eprintln!("[VARIANT] Room {} changed to {}", r.name, variant);
                    broadcast_state(r);
                });
            } else {
                let _ = tx_out.send(ServerToClient::Error {
                    message: "You must join a room before selecting game variant.".to_string(),
                });
            }
        }
        ClientToServer::JoinAsSpectator { room, name } => {
            let mut rooms = state.inner.lock();
            let r = rooms.entry(room.clone()).or_insert_with(|| game::Room::new(room.clone()));

            // Check if spectator already exists (shouldn't happen normally)
            if r.spectators.iter().any(|s| s.id == my_id) {
                let _ = tx_out.send(ServerToClient::Error {
                    message: "You are already spectating this room.".to_string(),
                });
                return;
            }

            // Add as spectator
            r.spectators.push(game::Spectator {
                id: my_id,
                name: name.clone(),
                tx: tx_out.clone(),
            });
            *joined_room = Some(room.clone());

            eprintln!("[SPECTATOR_JOIN] {} joined room {} as spectator", name, room);

            // Send the public room state to spectator
            let _ = tx_out.send(ServerToClient::SpectatorJoined {
                snapshot: game::public_room(r),
            });

            // Notify players that a spectator joined
            for p in r.players.iter() {
                let _ = p.tx.send(ServerToClient::Info {
                    message: format!("{} joined as spectator", name),
                });
            }
        }
        ClientToServer::LeaveSpectator => {
            if let Some(room) = joined_room {
                remove_spectator(state, room, my_id);
            }
        }
        ClientToServer::ElectToStart => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    handle_elect_to_start(r, my_id);
                });
            }
        }
        ClientToServer::DelegateDealer { player_id } => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    handle_delegate_dealer(r, my_id, player_id);
                });
            }
        }
        ClientToServer::ChooseGameVariant { variant } => {
            if let Some(room) = joined_room {
                with_room(state, room, |r| {
                    handle_choose_game_variant(r, my_id, variant);
                });
            }
        }
        ClientToServer::CreateTable { name, game_variant, ante, limit_small, limit_big, max_raises } => {
            handle_create_table(state, my_id, joined_room, tx_out, name, game_variant, ante, limit_small, limit_big, max_raises).await;
        }
        ClientToServer::PostComment { message } => {
            handle_post_comment(state.clone(), my_id, joined_room.clone(), message).await;
        }
        ClientToServer::ContinueToNextGame => {
            handle_continue_to_next_game(state.clone(), my_id, joined_room.clone()).await;
        }
        ClientToServer::RegisterTable { name, game_variant, ante, limit_small, limit_big, max_raises, server_port, player_count } => {
            handle_register_table(state.clone(), name, game_variant, ante, limit_small, limit_big, max_raises, server_port, player_count).await;
        }
    }
}

fn with_room<F: FnOnce(&mut game::Room)>(state: &AppState, room: &str, f: F) {
    let mut rooms = state.inner.lock();
    if let Some(r) = rooms.get_mut(room) {
        f(r);
    }
}

async fn handle_chat_message(state: AppState, player_id: Uuid, joined_room: Option<String>, message: String, scope: MessageScope) {
    use chrono::Utc;

    // Get player name and determine room context
    let (player_name, room_name) = match scope {
        MessageScope::Match => {
            if let Some(room) = &joined_room {
                let rooms = state.inner.lock();
                if let Some(r) = rooms.get(room) {
                    if let Some(player) = r.players.iter().find(|p| p.id == player_id) {
                        (player.name.clone(), Some(room.clone()))
                    } else {
                        return; // Player not in room
                    }
                } else {
                    return; // Room not found
                }
            } else {
                return; // No room joined
            }
        },
        MessageScope::Group | MessageScope::Global => {
            // For Group/Global messages, we need to find the player's name from any room they might be in
            let rooms = state.inner.lock();
            let mut found_name = None;
            for (_, room) in rooms.iter() {
                if let Some(player) = room.players.iter().find(|p| p.id == player_id) {
                    found_name = Some(player.name.clone());
                    break;
                }
            }
            if let Some(name) = found_name {
                (name, None)
            } else {
                return; // Player not found in any room
            }
        },
        MessageScope::Private => {
            // Private messages should not go through this function
            return;
        }
    };

    // Create stored message for persistence
    let stored_message = StoredMessage {
        player_name: player_name.clone(),
        message: message.clone(),
        scope,
        room: room_name.clone(),
        timestamp: Utc::now().to_rfc3339(),
        recipient: None, // No recipient for regular chat messages
    };

    // Store message to disk
    if let Err(e) = state.message_store.store_message(&stored_message).await {
        eprintln!("Failed to store message: {}", e);
    }

    // Create chat message for broadcasting
    let chat_msg = ServerToClient::ChatMessage {
        player_name,
        message: message.clone(),
        scope,
        room: room_name.clone(),
        timestamp: stored_message.timestamp.clone(),
        recipient: None, // No recipient for regular chat messages
    };

    // Route message based on scope
    match scope {
        MessageScope::Match => {
            if let Some(room_name) = room_name {
                let rooms = state.inner.lock();
                if let Some(r) = rooms.get(&room_name) {
                    // Send to all players in the room
                    for p in r.players.iter() {
                        let _ = p.tx.send(chat_msg.clone());
                    }
                }
            }
        },
        MessageScope::Group => {
            // Send to all players on the server (all rooms)
            let rooms = state.inner.lock();
            for (_, room) in rooms.iter() {
                for p in room.players.iter() {
                    let _ = p.tx.send(chat_msg.clone());
                }
            }
        },
        MessageScope::Global => {
            // For now, Global scope behaves like Group scope
            // In the future, this could be extended to support cross-server messaging
            let rooms = state.inner.lock();
            for (_, room) in rooms.iter() {
                for p in room.players.iter() {
                    let _ = p.tx.send(chat_msg.clone());
                }
            }
        },
        MessageScope::Private => {
            // Private messages are handled by a separate function
            // This case shouldn't be reached in normal chat flow
        }
    }

    eprintln!("[CHAT:{:?}] {} says: {}", scope, stored_message.player_name, message);
}

async fn handle_private_message(state: AppState, sender_id: Uuid, recipient_id: Uuid, message: String) {
    use chrono::Utc;

    // Find sender name by searching all rooms
    let sender_name = {
        let rooms = state.inner.lock();
        let mut found_name = None;
        for (_, room) in rooms.iter() {
            if let Some(player) = room.players.iter().find(|p| p.id == sender_id) {
                found_name = Some(player.name.clone());
                break;
            }
        }
        match found_name {
            Some(name) => name,
            None => {
                eprintln!("[PRIVATE] Sender {} not found in any room", sender_id);
                return; // Sender not found
            }
        }
    };

    // Find recipient and send message
    let recipient_found = {
        let rooms = state.inner.lock();
        let mut found = false;
        for (_, room) in rooms.iter() {
            if let Some(recipient) = room.players.iter().find(|p| p.id == recipient_id) {
                // Create private message
                let private_msg = ServerToClient::ChatMessage {
                    player_name: sender_name.clone(),
                    message: message.clone(),
                    scope: MessageScope::Private,
                    room: None, // No room for private messages
                    timestamp: Utc::now().to_rfc3339(),
                    recipient: Some(recipient_id),
                };

                // Send to recipient
                let _ = recipient.tx.send(private_msg.clone());

                // Also send to sender for confirmation/history
                if let Some(sender) = room.players.iter().find(|p| p.id == sender_id) {
                    let _ = sender.tx.send(private_msg);
                }

                found = true;
                break;
            }
        }
        found
    };

    if recipient_found {
        // Create stored message for persistence
        let stored_message = StoredMessage {
            player_name: sender_name.clone(),
            message: message.clone(),
            scope: MessageScope::Private,
            room: None,
            timestamp: Utc::now().to_rfc3339(),
            recipient: Some(recipient_id),
        };

        // Store message to disk
        if let Err(e) = state.message_store.store_message(&stored_message).await {
            eprintln!("Failed to store private message: {}", e);
        }

        eprintln!("[PRIVATE] {} -> {}: {}", sender_name, recipient_id, message);
    } else {
        eprintln!("[PRIVATE] Recipient {} not found", recipient_id);
        // Send error message back to sender
        let rooms = state.inner.lock();
        for (_, room) in rooms.iter() {
            if let Some(sender) = room.players.iter().find(|p| p.id == sender_id) {
                let error_msg = ServerToClient::Error {
                    message: format!("Recipient not found: {}", recipient_id),
                };
                let _ = sender.tx.send(error_msg);
                break;
            }
        }
    }
}

fn remove_player(state: &AppState, room: &str, id: Uuid) {
    let mut rooms = state.inner.lock();
    if let Some(r) = rooms.get_mut(room) {
        r.players.retain(|p| p.id != id);
        if r.players.is_empty() {
            rooms.remove(room);
            return;
        }
        broadcast_state(r);
    }
}

fn remove_spectator(state: &AppState, room: &str, id: Uuid) {
    let mut rooms = state.inner.lock();
    if let Some(r) = rooms.get_mut(room) {
        if let Some(pos) = r.spectators.iter().position(|s| s.id == id) {
            let spectator = r.spectators.remove(pos);
            eprintln!("[SPECTATOR_LEAVE] {} left room {} as spectator", spectator.name, room);

            // Notify players that spectator left
            for p in r.players.iter() {
                let _ = p.tx.send(ServerToClient::Info {
                    message: format!("{} left as spectator", spectator.name),
                });
            }
        }
    }
}

fn start_hand(r: &mut Room) {
    eprintln!(
        "[DEAL] start_hand: players={} dealer_seat={} variant={}",
        r.players.len(),
        r.dealer_seat,
        r.game_variant
    );

    r.phase = Phase::Dealing;
    r.pot = (r.players.len() as u64) * r.ante;
    r.deck = Some(Deck::standard_shuffled());
    r.community_cards.clear();

    for p in r.players.iter_mut() {
        p.folded = false;
        p.standing = false;
        p.up_cards.clear();
        p.down_cards.clear();
        p.ready = false;
        p.committed_round = 0;
    }

    // Deal cards based on game variant
    match r.game_variant {
        GameVariant::SevenTwentySeven => {
            // Deal one up card and one down card to each player
            for p in r.players.iter_mut() {
                let up = r.deck.as_mut().unwrap().draw(true).unwrap();
                let down = r.deck.as_mut().unwrap().draw(false).unwrap();
                p.up_cards.push(up);
                p.down_cards.push(down);
                let _ = p.tx.send(ServerToClient::YourHand {
                    hand: PrivateHand {
                        down_cards: p.down_cards.clone(),
                    },
                });
            }
        }
        GameVariant::Omaha => {
            // Deal 4 hole cards (all face down) to each player
            for p in r.players.iter_mut() {
                for _ in 0..4 {
                    let card = r.deck.as_mut().unwrap().draw(false).unwrap();
                    p.down_cards.push(card);
                }
                let _ = p.tx.send(ServerToClient::YourHand {
                    hand: PrivateHand {
                        down_cards: p.down_cards.clone(),
                    },
                });
            }
            // Deal 3 community cards (the flop)
            for _ in 0..3 {
                let card = r.deck.as_mut().unwrap().draw(true).unwrap();
                r.community_cards.push(card);
            }
        }
        GameVariant::TexasHoldem => {
            // Deal 2 hole cards (both face down) to each player
            for p in r.players.iter_mut() {
                for _ in 0..2 {
                    let card = r.deck.as_mut().unwrap().draw(false).unwrap();
                    p.down_cards.push(card);
                }
                let _ = p.tx.send(ServerToClient::YourHand {
                    hand: PrivateHand {
                        down_cards: p.down_cards.clone(),
                    },
                });
            }
            // Deal 3 community cards (the flop)
            for _ in 0..3 {
                let card = r.deck.as_mut().unwrap().draw(true).unwrap();
                r.community_cards.push(card);
            }
        }
    }

    r.phase = Phase::Acting;
    r.round = 1;

    // Set initial game state based on variant
    if r.game_variant.uses_community_cards() {
        // Community card games start with betting
        r.in_betting = true;
        r.current_bet = 0;
        r.raises_made = 0;
        r.betting_started_seat = next_alive_left_of(r, r.dealer_seat);
        r.last_aggressor_seat = None;
        r.to_act_seat = r.betting_started_seat;
        for p in r.players.iter_mut() {
            p.committed_round = 0;
        }
    } else {
        // 7/27 starts with draw phase
        r.in_betting = false;
        r.dealer_seat = r.dealer_seat % r.players.len();
        r.draw_started_seat = game::next_alive_left_of(r, r.dealer_seat);
        r.to_act_seat = r.draw_started_seat;
        r.draw_acted = (0..r.players.len())
            .map(|i| {
                let p = &r.players[i];
                p.folded || p.standing
            })
            .collect();
    }

    broadcast_state(r);
    eprintln!(
        "[DEAL] -> phase={:?} round={} to_act_seat={} in_betting={} variant={}",
        r.phase, r.round, r.to_act_seat, r.in_betting, r.game_variant
    );
}

fn next_alive_left_of(r: &Room, from: usize) -> usize {
    let n = r.players.len();
    let mut i = (from + 1) % n;
    while r.players[i].folded {
        i = (i + 1) % n;
    }
    i
}

fn player_take_card(r: &mut Room, id: Uuid) {
    eprintln!("[DRAW] take_card request id={}", &id.to_string()[..8]);
    if r.phase != Phase::Acting {
        eprintln!("[DRAW] reject: phase={:?}", r.phase);
        return;
    }
    if r.in_betting {
        eprintln!("[DRAW] reject: currently in betting");
        return;
    }

    let seat = match game::seat_of(r, id) {
        Some(s) => s,
        None => {
            eprintln!("[DRAW] reject: seat_of failed");
            return;
        }
    };

    if r.to_act_seat != seat {
        eprintln!(
            "[DRAW] reject: not your turn (to_act={} you={})",
            r.to_act_seat, seat
        );
        return;
    }
    if r.players[seat].folded {
        eprintln!("[DRAW] reject: player folded");
        return;
    }
    if r.players[seat].standing {
        eprintln!("[DRAW] reject: player already standing");
        return;
    }

    // Check if player already has max cards for this variant
    let current_cards = r.players[seat].up_cards.len() + r.players[seat].down_cards.len();
    let max_cards = r.game_variant.max_cards_per_player();
    if current_cards >= max_cards {
        eprintln!(
            "[DRAW] reject: player already has max cards ({}/{})",
            current_cards, max_cards
        );
        return;
    }

    let deck = match r.deck.as_mut() {
        Some(d) => d,
        None => {
            eprintln!("[DRAW] reject: deck is None");
            return;
        }
    };

    if let Some(c) = deck.draw(false) {
        r.players[seat].down_cards.push(c);
        let _ = r.players[seat].tx.send(ServerToClient::YourHand {
            hand: PrivateHand {
                down_cards: r.players[seat].down_cards.clone(),
            },
        });
        eprintln!(
            "[DRAW] seat {} drew a card; down={}",
            seat,
            r.players[seat].down_cards.len()
        );
    } else {
        eprintln!("[DRAW] deck exhausted");
        // You may want to end the hand here; for now just return.
        return;
    }

    let sc = score_hand(&game::all_cards(&r.players[seat]));
    if sc.bust_27 {
        r.players[seat].folded = true;
        let _ = r.players[seat].tx.send(ServerToClient::Info {
            message: "Busted (>27). You fold.".into(),
        });
        eprintln!("[DRAW] seat {} busted and folds", seat);
        r.draw_acted[seat] = true;
        advance_after_draw_action(r);
        return;
    }

    r.draw_acted[seat] = true;
    advance_after_draw_action(r);
}

fn player_stand(r: &mut Room, id: Uuid) {
    eprintln!("[DRAW] stand request id={}", &id.to_string()[..8]);
    if r.phase != Phase::Acting {
        eprintln!("[DRAW] reject: phase={:?}", r.phase);
        return;
    }
    if r.in_betting {
        eprintln!("[DRAW] reject: currently in betting");
        return;
    }

    let seat = match game::seat_of(r, id) {
        Some(s) => s,
        None => {
            eprintln!("[DRAW] reject: seat_of failed");
            return;
        }
    };

    if r.to_act_seat != seat {
        eprintln!(
            "[DRAW] reject: not your turn (to_act={} you={})",
            r.to_act_seat, seat
        );
        return;
    }
    if r.players[seat].folded {
        eprintln!("[DRAW] reject: player folded");
        return;
    }
    if r.players[seat].standing {
        eprintln!("[DRAW] reject: already standing");
        return;
    }

    r.players[seat].standing = true;
    r.draw_acted[seat] = true;
    eprintln!("[DRAW] seat {} stands", seat);
    advance_after_draw_action(r);
}

fn player_fold(r: &mut Room, id: Uuid) {
    if r.phase != Phase::Acting {
        return;
    }
    let seat = match game::seat_of(r, id) {
        Some(s) => s,
        None => return,
    };
    if r.players[seat].folded {
        return;
    }

    r.players[seat].folded = true;
    r.draw_acted[seat] = true;
    if r.in_betting {
        r.betting_acted[seat] = true;
    }

    if game::alive_seats(r).len() <= 1 {
        award_last_player_and_reset(r);
        return;
    }

    if r.in_betting {
        advance_betting_turn(r);
    } else {
        advance_after_draw_action(r);
    }
}

/* ---------------- small helpers used above ---------------- */

fn seat_of(r: &Room, id: Uuid) -> Option<usize> {
    r.players.iter().position(|p| p.id == id)
}


fn alive_seats(r: &Room) -> Vec<(usize, &PlayerSeat)> {
    r.players
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.folded)
        .collect()
}

fn advance_after_draw_action(r: &mut Room) {
    // Everyone done with draw? → go to betting
    if r.players.iter().all(|p| p.folded || p.standing) {
        eprintln!("[DRAW] all done drawing → start_betting_round");
        start_betting_round(r);
        return;
    }

    let n = r.players.len();
    let mut found_next = None;
    for _ in 0..n {
        r.to_act_seat = (r.to_act_seat + 1) % n;
        let p = &r.players[r.to_act_seat];
        if !p.folded && !r.draw_acted[r.to_act_seat] && !p.standing {
            found_next = Some(r.to_act_seat);
            break;
        }
    }

    if let Some(next) = found_next {
        eprintln!("[DRAW] next to act → seat {}", next);
        broadcast_state(r);
    } else {
        eprintln!("[DRAW] draw loop complete → start_betting_round");
        start_betting_round(r);
    }
}
/* ---------------- betting flow ---------------- */

fn start_betting_round(r: &mut Room) {
    r.in_betting = true;
    r.current_bet = 0;
    r.raises_made = 0;
    r.betting_started_seat = next_alive_left_of(r, r.dealer_seat);
    r.last_aggressor_seat = None;
    for p in r.players.iter_mut() {
        p.committed_round = 0;
    }
    r.betting_acted = (0..r.players.len()).map(|i| r.players[i].folded).collect();
    r.to_act_seat = r.betting_started_seat;
    broadcast_state(r);
}


fn player_check(r: &mut Room, id: Uuid) {
    if !r.in_betting || r.phase != Phase::Acting {
        return;
    }
    let seat = match game::seat_of(r, id) {
        Some(s) => s,
        None => return,
    };
    if r.to_act_seat != seat || r.players[seat].folded {
        return;
    }
    if r.current_bet != 0 {
        return;
    } // cannot check facing a bet
    r.betting_acted[seat] = true;
    advance_betting_turn(r);
}

fn player_bet_or_raise(r: &mut Room, id: Uuid, is_raise: bool) {
    if !r.in_betting || r.phase != Phase::Acting {
        return;
    }
    let seat = match game::seat_of(r, id) {
        Some(s) => s,
        None => return,
    };
    if r.to_act_seat != seat || r.players[seat].folded {
        return;
    }

    let sz = game::bet_size_for_round(r);

    if r.current_bet == 0 {
        if is_raise {
            return;
        }
        game::commit(r, seat, sz);
        r.current_bet = sz;
        r.last_aggressor_seat = Some(seat);
        r.raises_made = 1;
        // Reset all acting status except for folded and current player
        for i in 0..r.betting_acted.len() {
            r.betting_acted[i] = r.players[i].folded;
        }
        r.betting_acted[seat] = true;
        advance_betting_turn(r);
    } else {
        if !is_raise || r.raises_made >= r.max_raises {
            return;
        }
        let new_bet = r.current_bet + sz;
        let to_put = new_bet - r.players[seat].committed_round;
        game::commit(r, seat, to_put);
        r.current_bet = new_bet;
        r.last_aggressor_seat = Some(seat);
        r.raises_made += 1;
        // Reset all acting status except for folded and current player
        for i in 0..r.betting_acted.len() {
            r.betting_acted[i] = r.players[i].folded;
        }
        r.betting_acted[seat] = true;
        advance_betting_turn(r);
    }
}

fn player_call(r: &mut Room, id: Uuid) {
    if !r.in_betting || r.phase != Phase::Acting {
        return;
    }
    let seat = match game::seat_of(r, id) {
        Some(s) => s,
        None => return,
    };
    if r.to_act_seat != seat || r.players[seat].folded {
        return;
    }
    if r.current_bet == 0 {
        return;
    }

    let need = r.current_bet - r.players[seat].committed_round;
    commit(r, seat, need);
    r.betting_acted[seat] = true;
    advance_betting_turn(r);
}

fn commit(r: &mut Room, seat: usize, amount: u64) {
    if amount == 0 {
        return;
    }
    let p = &mut r.players[seat];
    let pay = amount.min(p.chips);
    p.chips -= pay;
    p.committed_round += pay;
    r.pot += pay;
}

fn advance_betting_turn(r: &mut Room) {
    // Check if all alive players have acted
    let all_acted = (0..r.players.len()).all(|i| {
        r.players[i].folded || r.betting_acted[i]
    });

    if all_acted {
        end_betting_round(r);
        return;
    }

    // Otherwise, advance to next alive seat that hasn't acted
    let n = r.players.len();
    for _ in 0..n {
        r.to_act_seat = (r.to_act_seat + 1) % n;
        if !r.players[r.to_act_seat].folded && !r.betting_acted[r.to_act_seat] {
            break;
        }
    }
    broadcast_state(r);
}

fn end_betting_round(r: &mut Room) {
    r.in_betting = false;

    // If all remaining players are standing → showdown, else next draw round
    if r.players.iter().all(|p| p.folded || p.standing) {
        do_showdown(r);
        return;
    }

    // new draw round: only non-standing, non-folded act
    r.round += 1;
    r.draw_started_seat = game::next_alive_left_of(r, r.dealer_seat);
    r.to_act_seat = r.draw_started_seat;
    r.draw_acted = (0..r.players.len())
        .map(|i| {
            let p = &r.players[i];
            p.folded || p.standing
        })
        .collect();

    broadcast_state(r);
}

/* ---------------- showdown / payouts ---------------- */

fn do_showdown(r: &mut Room) {
    let evals: Vec<_> = r
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.folded)
        .map(|(i, p)| (i, score_hand(&game::all_cards(p))))
        .collect();

    // 7 pot
    let best7 = evals
        .iter()
        .filter_map(|(_, s)| s.best_under_7.map(|_| s.dist_to_7.unwrap()))
        .min_by(|a, b| a.partial_cmp(b).unwrap());

    let winners7: Vec<Uuid> = match best7 {
        Some(d) => evals
            .iter()
            .filter(|(_, s)| s.best_under_7.is_some() && (s.dist_to_7.unwrap() - d).abs() < 1e-6)
            .map(|(i, _)| r.players[*i].id)
            .collect(),
        None => vec![],
    };

    // 27 pot
    let best27 = evals
        .iter()
        .filter_map(|(_, s)| (!s.bust_27).then_some(s.dist_to_27.unwrap()))
        .min_by(|a, b| a.partial_cmp(b).unwrap());

    let winners27: Vec<Uuid> = match best27 {
        Some(d) => evals
            .iter()
            .filter(|(_, s)| !s.bust_27 && (s.dist_to_27.unwrap() - d).abs() < 1e-6)
            .map(|(i, _)| r.players[*i].id)
            .collect(),
        None => vec![],
    };

    // split pot: half for 7 winners (if any), remainder for 27 winners
    let mut payouts: Vec<(Uuid, u64)> = vec![];
    let half = r.pot / 2;
    let mut paid = 0;
    if !winners7.is_empty() {
        let each = half / (winners7.len() as u64);
        for id in &winners7 {
            payouts.push((*id, each));
            paid += each;
        }
    }
    let remaining = r.pot - paid;
    if !winners27.is_empty() {
        let each = remaining / (winners27.len() as u64);
        for id in &winners27 {
            payouts.push((*id, each));
        }
    }
    for (id, amt) in &payouts {
        if let Some(p) = r.players.iter_mut().find(|p| p.id == *id) {
            p.chips += *amt;
        }
    }

    reveal_and_reset(r, winners7, winners27);
}

fn award_last_player_and_reset(r: &mut Room) {
    if let Some((seat, _)) = alive_seats(r).first() {
        let id = r.players[*seat].id;
        if let Some(p) = r.players.iter_mut().find(|p| p.id == id) {
            p.chips += r.pot;
        }
    }
    reveal_and_reset(r, vec![], vec![]);
}

fn reveal_and_reset(r: &mut Room, winners7: Vec<Uuid>, winners27: Vec<Uuid>) {
    let reveal: Vec<(Uuid, Vec<Card>)> = r.players.iter().map(|p| (p.id, game::all_cards(p))).collect();
    for p in r.players.iter() {
        let _ = p.tx.send(ServerToClient::Showdown {
            winners7: winners7.clone(),
            winners27: winners27.clone(),
            payouts: vec![],
            reveal: reveal.clone(),
        });
    }

    // Rotate dealer to the next player (to the left)
    let old_dealer_seat = r.dealer_seat;
    r.dealer_seat = (r.dealer_seat + 1) % r.players.len();

    // Update current_dealer_id to match the rotated dealer_seat
    if let Some(new_dealer_id) = game::next_dealer_left_of(r, old_dealer_seat) {
        r.current_dealer_id = Some(new_dealer_id);

        // Notify all players about the new dealer
        let new_dealer_name = r.players.iter()
            .find(|p| p.id == new_dealer_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        for player in r.players.iter() {
            let _ = player.tx.send(ServerToClient::DealerDelegated {
                dealer_id: new_dealer_id,
                dealer_name: new_dealer_name.clone(),
            });
        }

        eprintln!("[DEALER_ROTATION] New dealer: {} (seat {})", new_dealer_name, r.dealer_seat);
    }

    // Reset dealer system state
    r.elected_players.clear();

    // Transition to Comments phase
    r.phase = Phase::Comments;

    // Reset game state
    r.pot = 0;
    r.deck = None;
    r.in_betting = false;
    r.current_bet = 0;
    r.round = 0;
    r.raises_made = 0;

    // Reset all player states for next game
    for player in r.players.iter_mut() {
        player.folded = false;
        player.standing = false;
        player.up_cards.clear();
        player.down_cards.clear();
        player.ready = false;
        player.committed_round = 0;
    }

    broadcast_state(r);
}

/* ---------------- public snapshot & broadcast ---------------- */


fn broadcast_state(r: &game::Room) {
    let snapshot = game::public_room(r);
    eprintln!(
        "[BROADCAST] phase={:?} round={} in_betting={} to_act={} players={}",
        r.phase,
        r.round,
        r.in_betting,
        r.to_act_seat,
        r.players.len()
    );
    for (i, p) in r.players.iter().enumerate() {
        if p.tx
            .send(ServerToClient::UpdateState {
                snapshot: snapshot.clone(),
            })
            .is_err()
        {
            eprintln!(
                "[BROADCAST] failed to send to seat={} id={}",
                i,
                &p.id.to_string()[..8]
            );
        }
    }

    // Also broadcast to spectators
    for (i, s) in r.spectators.iter().enumerate() {
        if s.tx
            .send(ServerToClient::UpdateState {
                snapshot: snapshot.clone(),
            })
            .is_err()
        {
            eprintln!(
                "[BROADCAST] failed to send to spectator={} id={}",
                i,
                &s.id.to_string()[..8]
            );
        }
    }
}
fn log_room(prefix: &str, r: &Room) {
    let names: Vec<String> = r
        .players
        .iter()
        .map(|p| format!("{}({})", p.name, &p.id.to_string()[..8]))
        .collect();
    eprintln!("[{prefix}] room={} players={}", r.name, names.join(", "));
}
fn send_state_to(r: &game::Room, pid: Uuid) {
    let snap = game::public_room(r);
    eprintln!(
        "[DIRECT] to={} players={}",
        &pid.to_string()[..8],
        snap.players.len()
    );
    if let Some(p) = r.players.iter().find(|p| p.id == pid) {
        let _ = p.tx.send(ServerToClient::UpdateState { snapshot: snap });
    }
}
fn send_err_to(r: &Room, pid: Uuid, msg: impl Into<String>) {
    let msg = msg.into();
    eprintln!("[server validation] {}", msg); // <--- ADD THIS LINE
    if let Some(p) = r.players.iter().find(|p| p.id == pid) {
        let _ = p.tx.send(ServerToClient::Error { message: msg });
    }
}

fn handle_elect_to_start(r: &mut Room, player_id: Uuid) {
    // Verify player is in the room
    if game::seat_of(r, player_id).is_none() {
        send_err_to(r, player_id, "You must be in the room to elect to start.");
        return;
    }

    // Check minimum 4 players requirement
    if r.players.len() < 4 {
        send_err_to(r, player_id, "Minimum 4 players required to start game.");
        return;
    }

    // Only allow election in Lobby phase
    if r.phase != Phase::Lobby {
        send_err_to(r, player_id, "Can only elect to start when in lobby phase.");
        return;
    }

    // Add player to elected list if not already there
    if !r.elected_players.contains(&player_id) {
        r.elected_players.push(player_id);
        eprintln!("[DEALER] Player {} elected to start ({}/{})", &player_id.to_string()[..8], r.elected_players.len(), r.players.len());
    }

    // Check if all players have elected
    if r.elected_players.len() == r.players.len() {
        eprintln!("[DEALER] All players elected, moving to dealer selection phase");
        r.phase = Phase::DealerSelection;
    }

    // Broadcast updated state
    broadcast_to_room(r);
}

fn handle_delegate_dealer(r: &mut Room, requesting_player_id: Uuid, dealer_id: Uuid) {
    // Verify requesting player is in the room
    if game::seat_of(r, requesting_player_id).is_none() {
        send_err_to(r, requesting_player_id, "You must be in the room to delegate dealer.");
        return;
    }

    // Only allow dealer delegation in DealerSelection phase
    if r.phase != Phase::DealerSelection {
        send_err_to(r, requesting_player_id, "Can only delegate dealer during dealer selection phase.");
        return;
    }

    // Verify the proposed dealer is in the room
    if game::seat_of(r, dealer_id).is_none() {
        send_err_to(r, requesting_player_id, "Proposed dealer is not in this room.");
        return;
    }

    // Set the dealer
    r.current_dealer_id = Some(dealer_id);
    r.phase = Phase::GameSelection;

    eprintln!("[DEALER] Dealer delegated to {}, moving to game selection phase", &dealer_id.to_string()[..8]);

    // Send notification to all players
    if let Some(dealer) = r.players.iter().find(|p| p.id == dealer_id) {
        for player in &r.players {
            let _ = player.tx.send(ServerToClient::DealerDelegated {
                dealer_id,
                dealer_name: dealer.name.clone(),
            });
        }
    }

    // Broadcast updated state
    broadcast_to_room(r);
}

fn handle_choose_game_variant(r: &mut Room, player_id: Uuid, variant: GameVariant) {
    // Verify player is in the room
    if game::seat_of(r, player_id).is_none() {
        send_err_to(r, player_id, "You must be in the room to choose game variant.");
        return;
    }

    // Only allow game selection in GameSelection phase
    if r.phase != Phase::GameSelection {
        send_err_to(r, player_id, "Can only choose game variant during game selection phase.");
        return;
    }

    // Verify this player is the designated dealer
    if r.current_dealer_id != Some(player_id) {
        send_err_to(r, player_id, "Only the designated dealer can choose the game variant.");
        return;
    }

    // Set the game variant
    r.game_variant = variant;

    eprintln!("[DEALER] Game variant selected: {:?}, starting game", variant);

    // Send notification to all players
    if let Some(dealer) = r.players.iter().find(|p| p.id == player_id) {
        for player in &r.players {
            let _ = player.tx.send(ServerToClient::GameVariantSelected {
                variant,
                selected_by: dealer.name.clone(),
            });
        }
    }

    // Reset election state for next game
    r.elected_players.clear();

    // Move to dealing phase and start the game
    r.phase = Phase::Dealing;
    start_new_hand(r);

    // Broadcast updated state
    broadcast_to_room(r);
}

async fn handle_create_table(
    state: &AppState,
    creator_id: Uuid,
    joined_room: &mut Option<String>,
    tx_out: &mpsc::UnboundedSender<ServerToClient>,
    name: String,
    game_variant: GameVariant,
    ante: u64,
    limit_small: u64,
    limit_big: u64,
    max_raises: u32,
) {
    // Validate table name
    let trimmed_name = name.trim();
    if trimmed_name.is_empty() {
        let _ = tx_out.send(ServerToClient::Error {
            message: "Table name cannot be empty".to_string(),
        });
        return;
    }

    // Validate table configuration
    if ante == 0 || limit_small == 0 || limit_big == 0 || max_raises == 0 {
        let _ = tx_out.send(ServerToClient::Error {
            message: "Table configuration values must be greater than 0".to_string(),
        });
        return;
    }

    if limit_big <= limit_small {
        let _ = tx_out.send(ServerToClient::Error {
            message: "Big limit must be greater than small limit".to_string(),
        });
        return;
    }

    let mut rooms = state.inner.lock();

    // Check if table already exists
    if rooms.contains_key(trimmed_name) {
        let _ = tx_out.send(ServerToClient::Error {
            message: format!("Table '{}' already exists", trimmed_name),
        });
        return;
    }

    // Create new room with custom configuration
    let mut new_room = game::Room::new(trimmed_name.to_string());
    new_room.game_variant = game_variant;
    new_room.ante = ante;
    new_room.limit_small = limit_small;
    new_room.limit_big = limit_big;
    new_room.max_raises = max_raises;

    rooms.insert(trimmed_name.to_string(), new_room);
    drop(rooms); // Release the lock

    eprintln!("[CREATE_TABLE] Table '{}' created by {}", trimmed_name, &creator_id.to_string()[..8]);

    // Send confirmation to creator
    let _ = tx_out.send(ServerToClient::Info {
        message: format!("Table '{}' created successfully!", trimmed_name),
    });
}

fn broadcast_to_room(r: &game::Room) {
    broadcast_state(r);
}

fn start_new_hand(r: &mut game::Room) {
    start_hand(r);
}

async fn handle_post_comment(state: AppState, player_id: Uuid, joined_room: Option<String>, message: String) {
    use chrono::Utc;

    let room = match joined_room {
        Some(r) => r,
        None => return,
    };

    let mut rooms = state.inner.lock();
    let room_obj = match rooms.get_mut(&room) {
        Some(r) => r,
        None => return,
    };

    // Only allow comments in Comments phase
    if room_obj.phase != cctmog_protocol::Phase::Comments {
        return;
    }

    // Find the player name
    let player_name = match room_obj.players.iter().find(|p| p.id == player_id) {
        Some(p) => p.name.clone(),
        None => return,
    };

    // Create the comment
    let comment = cctmog_protocol::GameComment {
        player_id,
        player_name,
        message,
        timestamp: Utc::now().to_rfc3339(),
    };

    // Broadcast the comment to all players in the room
    for player in &room_obj.players {
        let _ = player.tx.send(cctmog_protocol::ServerToClient::GameComment {
            comment: comment.clone(),
        });
    }
}

async fn handle_continue_to_next_game(state: AppState, player_id: Uuid, joined_room: Option<String>) {
    let room = match joined_room {
        Some(r) => r,
        None => return,
    };

    with_room(&state, &room, |r| {
        // Only allow this action in Comments phase
        if r.phase != cctmog_protocol::Phase::Comments {
            return;
        }

        // Find the player and mark them as ready to continue
        if let Some(player) = r.players.iter_mut().find(|p| p.id == player_id) {
            player.ready = true;
        }

        // Check if all players are ready to continue
        let all_ready = r.players.iter().all(|p| p.ready);

        if all_ready {
            // Transition to the appropriate next phase
            if r.players.len() >= 4 {
                r.phase = cctmog_protocol::Phase::WaitingForDealer;
            } else {
                r.phase = cctmog_protocol::Phase::Lobby;
            }

            // Reset ready states for next time
            for player in r.players.iter_mut() {
                player.ready = false;
            }

            broadcast_state(r);
        }
    });
}

async fn handle_register_table(state: AppState, name: String, game_variant: cctmog_protocol::GameVariant, _ante: u64, _limit_small: u64, _limit_big: u64, _max_raises: u32, server_port: u16, player_count: usize) {
    println!("[REGISTER] Distributed table '{}' on port {} with {} players", name, server_port, player_count);

    // Store the distributed table info in a registry
    // For now, we'll add it to a special registry in the state
    let table_info = cctmog_protocol::TableInfo {
        name: name.clone(),
        game_variant,
        player_count,
        phase: cctmog_protocol::Phase::Lobby,
        server_port: Some(server_port),
    };

    // Add to distributed tables registry
    {
        let mut distributed_tables = state.distributed_tables.lock();
        distributed_tables.insert(name.clone(), table_info);
    }

    println!("[REGISTER] Table '{}' registered in central server registry", name);
}

async fn handle_list_tables(state: AppState, tx_out: &tokio::sync::mpsc::UnboundedSender<cctmog_protocol::ServerToClient>) {
    let mut tables = Vec::new();

    // Add local tables (hosted on central server)
    {
        let rooms = state.inner.lock();
        for (name, room) in rooms.iter() {
            tables.push(cctmog_protocol::TableInfo {
                name: name.clone(),
                game_variant: room.game_variant,
                player_count: room.players.len(),
                phase: room.phase.clone(),
                server_port: None, // Central server tables have no port
            });
        }
    }

    // Add distributed tables
    {
        let distributed_tables = state.distributed_tables.lock();
        for table_info in distributed_tables.values() {
            tables.push(table_info.clone());
        }
    }

    let table_count = tables.len();
    let _ = tx_out.send(cctmog_protocol::ServerToClient::TableList { tables });
    println!("[LIST] Sent {} tables to client", table_count);
}
