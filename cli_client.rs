use std::io::{self, Write};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use serde_json;
use cctmog_protocol::{ClientToServer, ServerToClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎮 CCTMOG Poker CLI Client");
    println!("==========================");

    // Get player name
    print!("Enter your name: ");
    io::stdout().flush()?;
    let mut player_name = String::new();
    io::stdin().read_line(&mut player_name)?;
    let player_name = player_name.trim().to_string();

    if player_name.is_empty() {
        println!("❌ Name cannot be empty");
        return Ok(());
    }

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

    // Handle incoming messages
    tokio::spawn({
        let player_name = player_name.clone();
        async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(server_msg) = serde_json::from_str::<ServerToClient>(&text) {
                            handle_server_message(server_msg, &player_name).await;
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
        }
    });

    println!("\n📋 Commands available:");
    println!("  ready     - Mark yourself as ready");
    println!("  take      - Take a card");
    println!("  stand     - Stand (no more cards)");
    println!("  fold      - Fold your hand");
    println!("  check     - Check (no bet)");
    println!("  bet       - Bet chips");
    println!("  call      - Call current bet");
    println!("  raise     - Raise the bet");
    println!("  chat <msg> - Send a chat message");
    println!("  quit      - Exit the game");
    println!("\nType commands and press Enter:");

    // Handle user input
    let stdin = tokio::io::stdin();
    let mut lines = BufReader::new(stdin).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim();

        if line == "quit" {
            break;
        }

        if let Some(msg) = parse_command(line) {
            let json = serde_json::to_string(&msg)?;
            write.send(Message::Text(json)).await?;
        } else {
            println!("❓ Unknown command: {}", line);
        }
    }

    println!("👋 Goodbye!");
    Ok(())
}

async fn handle_server_message(msg: ServerToClient, player_name: &str) {
    match msg {
        ServerToClient::Hello { your_id } => {
            println!("👋 Welcome! Your ID: {}", your_id);
        }
        ServerToClient::Joined { snapshot, your_seat, your_hand } => {
            println!("🎯 Joined game! You are in seat {}", your_seat);
            println!("🃏 Your hand: {} down cards", your_hand.down_cards.len());
            print_game_state(&snapshot);
        }
        ServerToClient::UpdateState { snapshot } => {
            print_game_state(&snapshot);
        }
        ServerToClient::YourHand { hand } => {
            println!("🃏 Your cards updated: {} down cards", hand.down_cards.len());
        }
        ServerToClient::Error { message } => {
            println!("❌ Error: {}", message);
        }
        ServerToClient::Info { message } => {
            println!("ℹ️  {}", message);
        }
        ServerToClient::Showdown { winners7, winners27, payouts, reveal } => {
            println!("\n🎭 SHOWDOWN!");
            println!("🏆 7-or-under winners: {:?}", winners7);
            println!("🏆 27-or-under winners: {:?}", winners27);
            println!("💰 Payouts: {:?}", payouts);
            for (player_id, cards) in reveal {
                println!("🃏 Player {}: {:?}", player_id, cards);
            }
        }
        ServerToClient::ChatMessage { player_name: sender, message, scope, .. } => {
            println!("💬 [{:?}] {}: {}", scope, sender, message);
        }
        _ => {
            println!("📨 Server message: {:?}", msg);
        }
    }
}

fn print_game_state(snapshot: &cctmog_protocol::PublicRoom) {
    println!("\n🎲 === GAME STATE ===");
    println!("🏠 Room: {}", snapshot.room);
    println!("🎯 Game: {}", snapshot.game_variant);
    println!("🕹️  Phase: {:?}", snapshot.phase);
    println!("💰 Pot: {} chips", snapshot.pot);
    println!("🎰 Round: {}", snapshot.round);

    if snapshot.in_betting {
        println!("💵 Current bet: {} chips", snapshot.current_bet);
    }

    println!("👥 Players ({}):", snapshot.players.len());
    for (i, player) in snapshot.players.iter().enumerate() {
        let status = if player.folded { " [FOLDED]" }
                    else if player.standing { " [STANDING]" }
                    else if player.ready { " [READY]" }
                    else { "" };

        let to_act = if i == snapshot.to_act_seat { " 👈 TO ACT" } else { "" };
        let dealer = if i == snapshot.dealer_seat { " 🎩 DEALER" } else { "" };

        println!("  {}: {} ({} chips, {} cards){}{}{}",
                 i, player.name, player.chips, player.cards_count,
                 status, to_act, dealer);

        if !player.up_cards.is_empty() {
            println!("    Up cards: {:?}", player.up_cards);
        }
    }
    println!("==================\n");
}

fn parse_command(input: &str) -> Option<ClientToServer> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    match parts[0].to_lowercase().as_str() {
        "ready" => Some(ClientToServer::SitReady),
        "take" => Some(ClientToServer::TakeCard),
        "stand" => Some(ClientToServer::Stand),
        "fold" => Some(ClientToServer::Fold),
        "check" => Some(ClientToServer::Check),
        "bet" => Some(ClientToServer::Bet),
        "call" => Some(ClientToServer::Call),
        "raise" => Some(ClientToServer::Raise),
        "chat" => {
            if parts.len() > 1 {
                let message = parts[1..].join(" ");
                Some(ClientToServer::Chat {
                    message,
                    scope: cctmog_protocol::MessageScope::Match
                })
            } else {
                None
            }
        }
        _ => None,
    }
}