use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Response,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
};
use tokio::sync::mpsc;
use uuid::Uuid;

use cctmog_protocol::{ClientToServer, ServerToClient, Phase, PrivateHand, StoredMessage};

// Re-use the game logic from the server
use crate::game;

#[derive(Debug, Clone)]
pub struct EmbeddedServerState {
    pub inner: Arc<Mutex<HashMap<String, game::Room>>>,
    pub players: Arc<Mutex<HashMap<Uuid, PlayerInfo>>>,
    pub port: u16,
}

#[derive(Debug)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub name: String,
    pub tx: mpsc::UnboundedSender<ServerToClient>,
    pub joined_room: Option<String>,
}

impl EmbeddedServerState {
    pub fn new(port: u16) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            players: Arc::new(Mutex::new(HashMap::new())),
            port,
        }
    }

    async fn register_table_with_central_server(&self, name: &str, game_variant: cctmog_protocol::GameVariant, ante: u64, limit_small: u64, limit_big: u64, max_raises: u32) {
        // Connect to central server and register this table
        let central_server_url = "ws://127.0.0.1:9001/ws";
        println!("[EMBEDDED] Connecting to central server at {}", central_server_url);

        match tokio_tungstenite::connect_async(central_server_url).await {
            Ok((mut ws, _)) => {
                println!("[EMBEDDED] Connected to central server successfully");
                let register_msg = cctmog_protocol::ClientToServer::RegisterTable {
                    name: name.to_string(),
                    game_variant,
                    ante,
                    limit_small,
                    limit_big,
                    max_raises,
                    server_port: self.port,
                    player_count: 1, // Start with 1 player (the creator)
                };

                let msg_json = serde_json::to_string(&register_msg).unwrap();
                println!("[EMBEDDED] Sending registration message: {}", msg_json);

                match ws.send(tokio_tungstenite::tungstenite::Message::Text(msg_json)).await {
                    Ok(_) => {
                        println!("[EMBEDDED] Registration message sent, waiting for acknowledgment...");
                        // Wait for a response to ensure the message was received
                        match tokio::time::timeout(std::time::Duration::from_secs(5), ws.next()).await {
                            Ok(Some(Ok(response))) => {
                                println!("[EMBEDDED] Received response: {:?}", response);
                                println!("[EMBEDDED] Registered table '{}' with central server", name);
                            }
                            Ok(Some(Err(e))) => {
                                println!("[EMBEDDED] Error in response: {}", e);
                            }
                            Ok(None) => {
                                println!("[EMBEDDED] Connection closed by central server");
                            }
                            Err(_) => {
                                println!("[EMBEDDED] Timeout waiting for response, but registration message was sent");
                            }
                        }
                    }
                    Err(e) => println!("[EMBEDDED] Failed to send registration message: {}", e),
                }
            }
            Err(e) => {
                println!("[EMBEDDED] Failed to connect to central server for table registration: {}", e);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmbeddedServer {
    state: EmbeddedServerState,
    port: u16,
    // Removed handle to make it cloneable - we'll manage the server lifecycle differently
}

impl EmbeddedServer {
    pub fn new(port: u16) -> Self {
        Self {
            state: EmbeddedServerState::new(port),
            port,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app = Router::new()
            .route("/ws", get(websocket_handler))
            .with_state(self.state.clone());

        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let listener = tokio::net::TcpListener::bind(addr).await?;

        println!("🔧 Embedded server listening on {}", addr);

        // Start the server and run indefinitely
        axum::serve(listener, app).await?;
        Ok(())
    }

    pub fn port(&self) -> u16 {
        self.port
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<EmbeddedServerState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: EmbeddedServerState) {
    let player_id = Uuid::new_v4();
    let (mut sender, mut receiver) = socket.split();
    let (tx_out, mut rx_out) = mpsc::unbounded_channel::<ServerToClient>();

    // Spawn task to handle outgoing messages
    let tx_task = tokio::spawn(async move {
        while let Some(msg) = rx_out.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    // Add player to state
    {
        let mut players = state.players.lock();
        players.insert(player_id, PlayerInfo {
            id: player_id,
            name: String::new(),
            tx: tx_out.clone(),
            joined_room: None,
        });
    }

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientToServer>(&text) {
                    handle_client_message(state.clone(), player_id, client_msg, &tx_out).await;
                }
            }
            Ok(Message::Close(_)) => break,
            _ => {}
        }
    }

    // Clean up player
    {
        let mut players = state.players.lock();
        players.remove(&player_id);
    }

    tx_task.abort();
}

async fn handle_client_message(
    state: EmbeddedServerState,
    player_id: Uuid,
    msg: ClientToServer,
    tx_out: &mpsc::UnboundedSender<ServerToClient>,
) {
    match msg {
        ClientToServer::Join { room, name } => {
            println!("[EMBEDDED] Player {} (id={}) joining room '{}'", name, &player_id.to_string()[..8], room);

            // Update player info
            {
                let mut players = state.players.lock();
                if let Some(player) = players.get_mut(&player_id) {
                    player.name = name.clone();
                    player.joined_room = Some(room.clone());
                }
            }

            // Create room if it doesn't exist, or join existing room
            let (snapshot, seat, should_broadcast) = {
                let mut rooms = state.inner.lock();
                let game_room = rooms.entry(room.clone()).or_insert_with(|| {
                    println!("[EMBEDDED] Creating new game room '{}'", room);
                    let mut new_room = game::Room::new(room.clone());
                    new_room.phase = Phase::DealerSelection;  // Start at dealer selection
                    new_room
                });

                // Check if player is already in the room
                let already_joined = game_room.players.iter().any(|p| p.id == player_id);

                if already_joined {
                    println!("[EMBEDDED] Player {} already in room, skipping duplicate join", name);
                    // Find their seat
                    let seat = game_room.players.iter().position(|p| p.id == player_id).unwrap();
                    (game_room.public_snapshot(), seat, false)
                } else {
                    // Add player to room
                    let seat = game_room.add_player(player_id, name.clone(), tx_out.clone());
                    println!("[EMBEDDED] Player {} added to room '{}' at seat {}", name, room, seat);
                    (game_room.public_snapshot(), seat, true)
                }
            };

            // Send joined confirmation to this player
            let _ = tx_out.send(ServerToClient::Joined {
                snapshot: snapshot.clone(),
                your_seat: seat,
                your_hand: PrivateHand { down_cards: vec![] },
            });

            // Broadcast state update to all other players if this was a new join
            if should_broadcast {
                let rooms = state.inner.lock();
                if let Some(game_room) = rooms.get(&room) {
                    for (i, player) in game_room.players.iter().enumerate() {
                        if player.id != player_id {
                            let _ = player.tx.send(ServerToClient::UpdateState {
                                snapshot: snapshot.clone(),
                            });
                        }
                    }
                }
            }

            println!("[EMBEDDED] Player at seat {} joined successfully, phase: {:?}", seat, Phase::DealerSelection);
        }

        ClientToServer::Chat { message, scope } => {
            println!("[EMBEDDED] Chat message from player: {}", message);

            // Get player info
            let player_name = {
                let players = state.players.lock();
                players.get(&player_id)
                    .map(|p| p.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            };

            let joined_room = {
                let players = state.players.lock();
                players.get(&player_id)
                    .and_then(|p| p.joined_room.clone())
            };

            if let Some(room_name) = joined_room {
                let mut rooms = state.inner.lock();
                if let Some(game_room) = rooms.get_mut(&room_name) {
                    // Store message in room
                    use chrono::Utc;
                    let stored_msg = StoredMessage {
                        player_name: player_name.clone(),
                        message: message.clone(),
                        scope,
                        room: Some(room_name.clone()),
                        timestamp: Utc::now().to_rfc3339(),
                        recipient: None,
                    };
                    game_room.chat_messages.push(stored_msg);

                    // Broadcast to all players in the room
                    let chat_msg = ServerToClient::ChatMessage {
                        player_name,
                        message,
                        scope,
                        room: Some(room_name),
                        timestamp: Utc::now().to_rfc3339(),
                        recipient: None,
                    };

                    for player in &game_room.players {
                        let _ = player.tx.send(chat_msg.clone());
                    }
                }
            }
        }

        ClientToServer::CreateTable { name, game_variant, ante, limit_small, limit_big, max_raises } => {
            let trimmed_name = name.trim();
            if trimmed_name.is_empty() {
                let _ = tx_out.send(ServerToClient::Error {
                    message: "Table name cannot be empty".to_string(),
                });
                return;
            }

            // Check and create room in a block to ensure mutex is dropped before await
            {
                let mut rooms = state.inner.lock();
                if rooms.contains_key(trimmed_name) {
                    let _ = tx_out.send(ServerToClient::Error {
                        message: format!("Table '{}' already exists", trimmed_name),
                    });
                    return;
                }

                // Create new room
                let mut new_room = game::Room::new(trimmed_name.to_string());
                new_room.game_variant = game_variant;
                new_room.ante = ante;
                new_room.limit_small = limit_small;
                new_room.limit_big = limit_big;
                new_room.max_raises = max_raises;

                rooms.insert(trimmed_name.to_string(), new_room);
            } // Mutex is automatically dropped here

            println!("[EMBEDDED] Table '{}' created by {}", trimmed_name, &player_id.to_string()[..8]);

            // Register table with central server for discovery
            println!("[EMBEDDED] Attempting to register table '{}' with central server", trimmed_name);
            state.register_table_with_central_server(trimmed_name, game_variant, ante, limit_small, limit_big, max_raises).await;

            let _ = tx_out.send(ServerToClient::Info {
                message: format!("Table '{}' created successfully on your local server!", trimmed_name),
            });
        }
        _ => {
            // For other messages, we can implement them later or delegate to main server logic
            let _ = tx_out.send(ServerToClient::Error {
                message: "Feature not yet implemented in embedded server".to_string(),
            });
        }
    }
}