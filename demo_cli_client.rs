use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use cctmog_protocol::{ClientToServer, ServerToClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get player name from command line argument or use default
    let args: Vec<String> = env::args().collect();
    let player_name = if args.len() > 1 {
        args[1].clone()
    } else {
        format!("Player{}", std::process::id())
    };

    println!("🎮 CCTMOG Poker CLI Client");
    println!("==========================");
    println!("Player: {}", player_name);

    // Connect to server
    let url = "ws://127.0.0.1:9001/ws";
    println!("🔗 Connecting to {}...", url);

    let (ws_stream, _) = connect_async(url).await?;
    println!("✅ Connected to server!");

    let (mut write, mut read) = ws_stream.split();

    // Join room
    let room_name = "shared_game_room".to_string();
    let join_msg = ClientToServer::Join {
        room: room_name.clone(),
        name: player_name.clone(),
    };

    let join_json = serde_json::to_string(&join_msg)?;
    write.send(Message::Text(join_json)).await?;

    println!("🚪 Joining room '{}'...", room_name);

    // Auto-ready after joining
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    let ready_msg = ClientToServer::SitReady;
    let ready_json = serde_json::to_string(&ready_msg)?;
    write.send(Message::Text(ready_json)).await?;
    println!("✅ Marked as ready!");

    // Handle incoming messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(server_msg) = serde_json::from_str::<ServerToClient>(&text) {
                    handle_server_message(server_msg.clone(), &player_name).await;

                    // Auto-play some moves
                    if let Some(response) = auto_play_response(&server_msg).await {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        let response_json = serde_json::to_string(&response)?;
                        write.send(Message::Text(response_json)).await?;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("🔌 Connection closed by server");
                break;
            }
            Err(e) => {
                println!("❌ WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    println!("👋 {} disconnected!", player_name);
    Ok(())
}

async fn handle_server_message(msg: ServerToClient, player_name: &str) {
    match msg {
        ServerToClient::Hello { your_id } => {
            println!("👋 [{}] Welcome! Your ID: {}", player_name, your_id);
        }
        ServerToClient::Joined { snapshot, your_seat, your_hand } => {
            println!("🎯 [{}] Joined game! You are in seat {}", player_name, your_seat);
            println!("🃏 [{}] Your hand: {} down cards", player_name, your_hand.down_cards.len());
            print_game_state(&snapshot, player_name);
        }
        ServerToClient::UpdateState { snapshot } => {
            print_game_state(&snapshot, player_name);
        }
        ServerToClient::YourHand { hand } => {
            println!("🃏 [{}] Your cards updated: {} down cards", player_name, hand.down_cards.len());
        }
        ServerToClient::Error { message } => {
            println!("❌ [{}] Error: {}", player_name, message);
        }
        ServerToClient::Info { message } => {
            println!("ℹ️  [{}] {}", player_name, message);
        }
        ServerToClient::Showdown { winners7, winners27, payouts, reveal } => {
            println!("\n🎭 [{}] SHOWDOWN!", player_name);
            println!("🏆 [{}] 7-or-under winners: {:?}", player_name, winners7);
            println!("🏆 [{}] 27-or-under winners: {:?}", player_name, winners27);
            println!("💰 [{}] Payouts: {:?}", player_name, payouts);
            for (player_id, cards) in reveal {
                println!("🃏 [{}] Player {}: {:?}", player_name, player_id, cards);
            }
        }
        ServerToClient::ChatMessage { player_name: sender, message, scope, .. } => {
            println!("💬 [{}] [{:?}] {}: {}", player_name, scope, sender, message);
        }
        _ => {
            println!("📨 [{}] Server message: {:?}", player_name, msg);
        }
    }
}

fn print_game_state(snapshot: &cctmog_protocol::PublicRoom, player_name: &str) {
    println!("\n🎲 [{}] === GAME STATE ===", player_name);
    println!("🏠 [{}] Room: {}", player_name, snapshot.room);
    println!("🎯 [{}] Game: {}", player_name, snapshot.game_variant);
    println!("🕹️  [{}] Phase: {:?}", player_name, snapshot.phase);
    println!("💰 [{}] Pot: {} chips", player_name, snapshot.pot);
    println!("🎰 [{}] Round: {}", player_name, snapshot.round);

    if snapshot.in_betting {
        println!("💵 [{}] Current bet: {} chips", player_name, snapshot.current_bet);
    }

    println!("👥 [{}] Players ({}):", player_name, snapshot.players.len());
    for (i, player) in snapshot.players.iter().enumerate() {
        let status = if player.folded { " [FOLDED]" }
                    else if player.standing { " [STANDING]" }
                    else if player.ready { " [READY]" }
                    else { "" };

        let to_act = if i == snapshot.to_act_seat { " 👈 TO ACT" } else { "" };
        let dealer = if i == snapshot.dealer_seat { " 🎩 DEALER" } else { "" };

        println!("  [{}] {}: {} ({} chips, {} cards){}{}{}",
                 player_name, i, player.name, player.chips, player.cards_count,
                 status, to_act, dealer);

        if !player.up_cards.is_empty() {
            println!("    [{}] Up cards: {:?}", player_name, player.up_cards);
        }
    }
    println!("[{}] ==================\n", player_name);
}

async fn auto_play_response(msg: &ServerToClient) -> Option<ClientToServer> {
    match msg {
        ServerToClient::UpdateState { snapshot } => {
            // Auto-play during draw phase
            if snapshot.phase == cctmog_protocol::Phase::Acting && !snapshot.in_betting {
                // During drawing, sometimes take cards, sometimes stand
                if snapshot.round <= 2 {
                    Some(ClientToServer::TakeCard)
                } else if snapshot.round <= 4 {
                    Some(ClientToServer::TakeCard)
                } else {
                    Some(ClientToServer::Stand)
                }
            }
            // Auto-play during betting phase
            else if snapshot.phase == cctmog_protocol::Phase::Acting && snapshot.in_betting {
                if snapshot.current_bet == 0 {
                    Some(ClientToServer::Check)
                } else {
                    Some(ClientToServer::Call)
                }
            } else {
                None
            }
        }
        _ => None
    }
}