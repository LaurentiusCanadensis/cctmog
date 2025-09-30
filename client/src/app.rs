// client/src/app.rs
use std::time::Duration;
use iced::{Element, Length, Subscription, Task};
use iced_widget::{button, column, container, horizontal_rule, row, text, text_input, Space};

use uuid::Uuid;
use rand::Rng;
use cctmog_protocol::{ClientToServer, GameVariant, MessageScope, Phase, PublicRoom, ServerToClient};
use iced::Alignment;
use crate::messages::Msg;
use crate::{cards_row_svg, CardSize, render_action_bar};
use crate::ui::cards::face_down_cards_row;
use crate::ui::table::round_table_view;
use crate::ui::canvas::felt;
use crate::ui::ws::subscription; // <- bring ui::ws::subscription into scope
use crate::ui::views::{splash_view, name_input_view, table_choice_view, table_creation_view, table_browser_view, game_view, connect_overlay, comments_view};
use crate::ui::shared::{brand_logo, footer};

pub use crate::states::AppState;

#[derive(Clone)]
pub struct App {
    pub app_state: AppState,
    pub splash_start_time: Option<std::time::Instant>,
    pub url: String,
    pub name: String,
    pub room: String,

    pub connecting: bool,
    pub connected: bool,

    pub your_id: Option<Uuid>,
    pub your_seat: Option<usize>,
    pub your_hand: cctmog_protocol::PrivateHand,
    pub snapshot: Option<cctmog_protocol::PublicRoom>,

    pub tx_out: Option<iced::futures::channel::mpsc::UnboundedSender<ClientToServer>>,
    pub log: Vec<String>,
    pub show_asset_test: bool, // reused as "show log"
    pub auto_started: bool,

    // Chat state
    pub chat_messages: Vec<(String, String, MessageScope)>, // (player_name, message, scope)
    pub chat_input: String,
    pub chat_scope: MessageScope,

    // Table listing
    pub available_tables: Vec<cctmog_protocol::TableInfo>,
    // Name validation
    pub name_error: Option<String>,

    // Scheduling state
    pub schedule_time_input: String,

    // Table creation state
    pub table_name: String,
    pub table_game_variant: GameVariant,
    pub table_ante: String,
    pub table_limit_small: String,
    pub table_limit_big: String,
    pub table_max_raises: String,
    pub table_creation_error: Option<String>,
    pub pending_table_creation: Option<ClientToServer>,

    // Comments state
    pub comment_input: String,
    pub game_comments: Vec<cctmog_protocol::GameComment>,
    pub ready_to_continue: bool,

    // Embedded server state
    pub embedded_server: Option<crate::embedded_server::EmbeddedServer>,
    pub local_server_port: u16,

    // Window state
    pub window_size: Option<iced::Size>,

    // Dealer and game selection state
    pub selected_dealer: Option<String>,
    pub dealer_splash_start_time: Option<std::time::Instant>,

    // Host mode state
    pub is_hosting: bool,
    pub host_name: Option<String>,
    pub host_server_port: Option<u16>,
    pub waiting_for_players: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            app_state: AppState::Splash,
            splash_start_time: Some(std::time::Instant::now()),
            url: "ws://127.0.0.1:9001/ws".into(),
            name: "".into(),
            room: "room-1".into(),
            connecting: false,
            connected: false,
            your_id: None,
            your_seat: None,
            your_hand: cctmog_protocol::PrivateHand { down_cards: vec![] },
            snapshot: None,
            tx_out: None,
            log: Vec::new(),
            show_asset_test: false,
            auto_started: false,
            chat_messages: Vec::new(),
            chat_input: String::new(),
            chat_scope: MessageScope::Match,
            available_tables: Vec::new(),
            name_error: None,
            schedule_time_input: String::new(),

            // Table creation defaults
            table_name: String::new(),
            table_game_variant: GameVariant::SevenTwentySeven,
            table_ante: "10".to_string(),
            table_limit_small: "10".to_string(),
            table_limit_big: "20".to_string(),
            table_max_raises: "3".to_string(),
            table_creation_error: None,
            pending_table_creation: None,

            // Comments defaults
            comment_input: String::new(),
            game_comments: Vec::new(),
            ready_to_continue: false,

            // Embedded server defaults
            embedded_server: None,
            local_server_port: 0, // Will be assigned dynamically

            // Window defaults
            window_size: None,

            // Dealer and game selection defaults
            selected_dealer: None,
            dealer_splash_start_time: None,

            // Host mode defaults
            is_hosting: false,
            host_name: None,
            host_server_port: None,
            waiting_for_players: false,
        }
    }
}

impl App {
    pub fn check_for_available_host(&mut self) {
        // Check if there's a host announcement file
        if let Ok(host_info) = std::fs::read_to_string("/tmp/cctmog_host") {
            if let Some((name, port_str)) = host_info.trim().split_once(':') {
                if let Ok(port) = port_str.parse::<u16>() {
                    // Only update if we don't already have this info
                    if self.host_name.as_ref() != Some(&name.to_string()) || self.host_server_port != Some(port) {
                        self.host_name = Some(name.to_string());
                        self.host_server_port = Some(port);
                        self.log(format!("🔍 Found host: {} on port {}", name, port));
                    }
                }
            }
        }
    }

    pub fn log<S: Into<String>>(&mut self, s: S) {
        self.log.push(s.into());
        if self.log.len() > 400 {
            self.log.remove(0);
        }
    }

    pub fn send_message(&mut self, msg: ClientToServer) {
        if let Some(ref tx) = self.tx_out {
            if let Err(e) = tx.unbounded_send(msg) {
                self.log(format!("Failed to send message: {}", e));
            }
        } else {
            self.log("Cannot send message: not connected");
        }
    }

    async fn start_embedded_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.embedded_server.is_none() {
            // Find an available port starting from 9100
            let mut port = 9100;
            loop {
                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
                if tokio::net::TcpListener::bind(addr).await.is_ok() {
                    break;
                }
                port += 1;
                if port > 9999 {
                    return Err("No available ports in range 9100-9999".into());
                }
            }

            let mut server = crate::embedded_server::EmbeddedServer::new(port);
            server.start().await?;

            self.local_server_port = port;
            self.embedded_server = Some(server);
            self.log(format!("🔧 Started embedded server on port {}", port));
        }
        Ok(())
    }

    fn stop_embedded_server(&mut self) {
        if let Some(_server) = self.embedded_server.take() {
            // Note: In this simplified implementation, we don't have a direct way to stop the server
            // The server will stop when the tokio task is dropped/cancelled
            self.log("🔧 Embedded server reference removed");
        }
    }

    pub(crate) fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            // Handle splash screen timer
            Msg::Tick => {
                if self.app_state == AppState::Splash {
                    if let Some(start_time) = self.splash_start_time {
                        if start_time.elapsed() >= Duration::from_secs(3) {
                            self.app_state = AppState::NameInput;
                            self.splash_start_time = None;
                        }
                    }
                } else if self.app_state == AppState::DealerSplash {
                    if let Some(start_time) = self.dealer_splash_start_time {
                        if start_time.elapsed() >= Duration::from_millis(500) {
                            self.app_state = AppState::GameSelection;
                            self.dealer_splash_start_time = None;
                        }
                    }
                } else if self.app_state == AppState::Lounge && !self.is_hosting {
                    // Periodic host discovery for non-hosting clients
                    self.check_for_available_host();
                }
            }

            // Handle new screen transitions
            Msg::SplashFinished => {
                self.app_state = AppState::NameInput;
                self.splash_start_time = None;
            }

            Msg::CreateTable => {
                self.app_state = AppState::TableCreation;
                self.table_creation_error = None;
                // Default table name to user's name if empty
                if self.table_name.is_empty() && !self.name.is_empty() {
                    self.table_name = format!("{}'s Table", self.name);
                }
                // Note: No auto-connection here - user must explicitly connect
            }

            Msg::JoinTable => {
                // Quick join to shared game room
                self.room = "shared_game_room".to_string();
                self.app_state = AppState::ConnectOverlay;
                self.connecting = true;
            }

            Msg::BrowseTables => {
                self.app_state = AppState::TableBrowser;
                self.send(ClientToServer::ListTables);
            }

            Msg::CreateNewGame => {
                self.room = "table".to_string();
                self.app_state = AppState::ConnectOverlay;
                return self.update(Msg::ConnectToggle);
            }

            Msg::BackToHome => {
                // Reset connection state and go back to lounge
                self.app_state = AppState::Lounge;
                self.connecting = false;
                self.connected = false;
                self.tx_out = None;
                self.snapshot = None;
                self.your_id = None;
                self.your_seat = None;
                self.your_hand.down_cards.clear();
                self.auto_started = false;
            }

            Msg::ServerUrlChanged(s) => self.url = s,
            Msg::NameChanged(s) => {
                self.name = s;
                self.name_error = None;
            }
            Msg::RoomChanged(s) => self.room = s,

            Msg::ConnectToggle => {
                self.connecting = true;
                self.connected = false;
                self.tx_out = None;
                self.snapshot = None;
                self.your_hand.down_cards.clear();
                self.auto_started = false;
                self.log("connecting…");
            }

            Msg::WsConnected(tx) => {
                self.tx_out = Some(tx.clone());
                self.connected = true;
                self.connecting = false;
                self.log(format!("connected to {}", self.url));

                // Only auto-join if we're in a connecting state that expects to join a room
                if self.app_state == AppState::ConnectOverlay {
                    println!("🎮 Auto-joining game room: {}", self.room);
                    self.send(ClientToServer::Join {
                        room: self.room.clone(),
                        name: self.name.clone(),
                    });
                    self.log(format!("🎮 Joining game room: {}", self.room));
                }

                // Handle different states after WebSocket connection
                if self.app_state == AppState::TableBrowser {
                    self.send(ClientToServer::ListTables);
                    self.log("connected to table browser");
                } else if self.app_state == AppState::TableCreation {
                    // Send pending table creation request if present
                    if let Some(pending_request) = self.pending_table_creation.take() {
                        self.log(format!("sending table creation request: {:?}", pending_request));
                        let _ = tx.unbounded_send(pending_request);
                        self.app_state = AppState::TableChoice;
                        self.log("table creation request sent to embedded server");
                    } else {
                        self.log("connected for table creation but no pending request");
                    }
                } else if self.app_state == AppState::ConnectOverlay {
                    // Only transition to Game state if we're explicitly connecting to a game
                    self.app_state = AppState::Game;
                    self.log("connected to game");
                }
            }

            Msg::WsEvent(ev) => match ev {
                ServerToClient::Hello { your_id } => {
                    self.your_id = Some(your_id);
                    self.log(format!("hello: {}", your_id));
                }
                ServerToClient::Joined { snapshot, your_seat, your_hand } => {
                    println!("🎰 Joined as player in seat {}", your_seat);
                    self.snapshot = Some(snapshot);
                    self.your_seat = Some(your_seat);
                    self.your_hand = your_hand;
                    self.auto_started = false;
                    self.app_state = AppState::Game; // Now transition to Game state
                    self.log(format!("🎮 Joined communal game as player: seat {}", your_seat));
                    self.send(ClientToServer::SitReady);
                }
                ServerToClient::UpdateState { snapshot } => {
                    let names: Vec<String> = snapshot.players.iter()
                        .map(|p| format!("{}({})", p.name, &p.id.to_string()[..8]))
                        .collect();
                    self.log(format!(
                        "state: phase={:?}, round={}, pot={}, to_act={}, players={} [{}]",
                        snapshot.phase, snapshot.round, snapshot.pot,
                        snapshot.to_act_seat, snapshot.players.len(), names.join(", ")
                    ));

                    if snapshot.phase == Phase::Lobby {
                        let all_ready = snapshot.players.iter().all(|p| p.ready) && snapshot.players.len() >= 2;
                        if all_ready && !self.auto_started {
                            self.auto_started = true;
                            self.log("auto-starting hand…");
                            self.send(ClientToServer::StartHand);
                        }
                    } else {
                        self.auto_started = false;
                    }

                    // Handle phase transitions
                    if snapshot.phase == Phase::Comments && self.app_state == AppState::Game {
                        self.app_state = AppState::Comments;
                        self.game_comments.clear(); // Clear previous comments
                        self.ready_to_continue = false;
                    } else if snapshot.phase == Phase::Lobby && self.app_state == AppState::Comments {
                        self.app_state = AppState::Game;
                    }

                    self.snapshot = Some(snapshot);
                }
                ServerToClient::YourHand { hand } => {
                    self.log(format!("received your hand: {} down", hand.down_cards.len()));
                    self.your_hand = hand;
                }
                ServerToClient::Showdown { .. } => self.log("showdown"),
                ServerToClient::Error { message } => self.log(format!("server error: {message}")),
                ServerToClient::Info { message } => self.log(format!("info: {message}")),
                ServerToClient::ChatMessage { player_name, message, scope, room: _, timestamp: _, recipient: _ } => {
                    self.chat_messages.push((player_name, message, scope));
                }
                ServerToClient::TableList { tables } => {
                    self.available_tables = tables;
                }
                ServerToClient::SpectatorJoined { snapshot } => {
                    println!("👁️ Joined as spectator - table is full");
                    self.log("🎮 Joined communal game as spectator (table full)");
                    self.room = snapshot.room.clone();
                    self.snapshot = Some(snapshot);
                    self.app_state = AppState::Game;
                }
                ServerToClient::DealerDelegated { dealer_name, .. } => {
                    self.log(format!("Dealer delegated to {}", dealer_name));
                }
                ServerToClient::GameVariantSelected { variant, selected_by } => {
                    self.log(format!("Game variant {} selected by {}", variant, selected_by));
                }
                ServerToClient::GameComment { comment } => {
                    self.game_comments.push(comment);
                }
            },

            Msg::WsError(e) => {
                self.log(format!("[ws error] connecting to {} failed: {}", self.url, e));
                self.connected = false;
                self.tx_out = None;
            }

            Msg::SitReady => self.send(ClientToServer::SitReady),
            Msg::StartHand => self.send(ClientToServer::StartHand),
            Msg::TakeCard => {
                println!("🎯 TakeCard button clicked!");
                self.send(ClientToServer::TakeCard)
            },
            Msg::Stand   => {
                println!("🛑 Stand button clicked!");
                self.send(ClientToServer::Stand)
            },
            Msg::Fold    => self.send(ClientToServer::Fold),
            Msg::Check   => self.send(ClientToServer::Check),
            Msg::Bet     => self.send(ClientToServer::Bet),
            Msg::Call    => self.send(ClientToServer::Call),
            Msg::Raise   => self.send(ClientToServer::Raise),

            // Chat messages
            Msg::ChatInputChanged(input) => {
                self.chat_input = input;
            }
            Msg::SendChat => {
                if !self.chat_input.trim().is_empty() {
                    self.send(ClientToServer::Chat {
                        message: self.chat_input.clone(),
                        scope: self.chat_scope
                    });
                    self.chat_input.clear();
                }
            }

            // Join specific table
            Msg::JoinTableByName(table_name) => {
                // Find the table info to check if it's on a distributed server
                if let Some(table_info) = self.available_tables.iter().find(|t| t.name == table_name) {
                    // Check if this table is on a distributed server
                    if let Some(server_port) = table_info.server_port {
                        // Connect to distributed server
                        self.url = format!("ws://127.0.0.1:{}/ws", server_port);
                        self.log(format!("🔗 Connecting to distributed table on port {}", server_port));
                    } else {
                        // Connect to central server (default)
                        self.url = "ws://127.0.0.1:9001/ws".to_string();
                        self.log("🔗 Connecting to central server table");
                    }
                }

                self.room = table_name;
                self.app_state = AppState::ConnectOverlay;
                return self.update(Msg::ConnectToggle);
            }

            // Name confirmation
            Msg::ConfirmName => {
                let trimmed_name = self.name.trim();
                if trimmed_name.is_empty() {
                    self.name_error = Some("Please enter a name".to_string());
                } else if trimmed_name.len() < 2 {
                    self.name_error = Some("Name must be at least 2 characters".to_string());
                } else if trimmed_name.len() > 20 {
                    self.name_error = Some("Name must be 20 characters or less".to_string());
                } else if !trimmed_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                    self.name_error = Some("Name can only contain letters, numbers, _ and -".to_string());
                } else {
                    self.name_error = None;
                    self.app_state = AppState::Lounge;
                }
            }

            // Scheduling messages
            Msg::ScheduleTimeChanged(time) => {
                self.schedule_time_input = time;
            }
            Msg::ScheduleGame => {
                if !self.schedule_time_input.trim().is_empty() {
                    self.send(ClientToServer::ScheduleGame {
                        start_time: self.schedule_time_input.clone()
                    });
                    self.schedule_time_input.clear();
                }
            }
            Msg::CheckIn => {
                self.send(ClientToServer::CheckIn);
            }
            Msg::SelectGameVariant(variant) => {
                self.send(ClientToServer::SelectGameVariant { variant });
            }

            // Table creation form handlers
            Msg::TableNameChanged(name) => {
                self.table_name = name;
                self.table_creation_error = None;
            }
            Msg::TableGameVariantChanged(variant) => {
                self.table_game_variant = variant;
            }
            Msg::TableAnteChanged(ante) => {
                self.table_ante = ante;
                self.table_creation_error = None;
            }
            Msg::TableLimitSmallChanged(limit) => {
                self.table_limit_small = limit;
                self.table_creation_error = None;
            }
            Msg::TableLimitBigChanged(limit) => {
                self.table_limit_big = limit;
                self.table_creation_error = None;
            }
            Msg::TableMaxRaisesChanged(raises) => {
                self.table_max_raises = raises;
                self.table_creation_error = None;
            }
            Msg::SubmitTableCreation => {
                // Start embedded server first for table creation
                return Task::perform(
                    async move {},
                    |_| Msg::StartEmbeddedServerForTable
                );
            }

            Msg::StartEmbeddedServerForTable => {
                // Validate inputs
                let trimmed_name = self.table_name.trim();
                if trimmed_name.is_empty() {
                    self.table_creation_error = Some("Please enter a table name".to_string());
                    return Task::none();
                }

                // Parse numeric fields
                let ante = match self.table_ante.parse::<u64>() {
                    Ok(val) if val > 0 => val,
                    _ => {
                        self.table_creation_error = Some("Ante must be a positive number".to_string());
                        return Task::none();
                    }
                };

                let limit_small = match self.table_limit_small.parse::<u64>() {
                    Ok(val) if val > 0 => val,
                    _ => {
                        self.table_creation_error = Some("Small limit must be a positive number".to_string());
                        return Task::none();
                    }
                };

                let limit_big = match self.table_limit_big.parse::<u64>() {
                    Ok(val) if val > 0 => val,
                    _ => {
                        self.table_creation_error = Some("Big limit must be a positive number".to_string());
                        return Task::none();
                    }
                };

                if limit_big <= limit_small {
                    self.table_creation_error = Some("Big limit must be greater than small limit".to_string());
                    return Task::none();
                }

                let max_raises = match self.table_max_raises.parse::<u32>() {
                    Ok(val) if val > 0 => val,
                    _ => {
                        self.table_creation_error = Some("Max raises must be a positive number".to_string());
                        return Task::none();
                    }
                };

                // Start embedded server if not already running
                if self.embedded_server.is_none() {
                    self.log("🔧 Starting embedded server for table creation...".to_string());
                    return Task::perform(
                        async {
                            // Find an available port starting from 9100
                            let mut port = 9100;
                            loop {
                                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
                                if tokio::net::TcpListener::bind(addr).await.is_ok() {
                                    break;
                                }
                                port += 1;
                                if port > 9999 {
                                    return Err("No available ports in range 9100-9999".to_string());
                                }
                            }
                            Ok(port)
                        },
                        |result| match result {
                            Ok(port) => Msg::EmbeddedServerStarted(port),
                            Err(err) => Msg::EmbeddedServerError(err),
                        },
                    );
                }

                // Server is already running, proceed with table creation
                let create_table_cmd = cctmog_protocol::ClientToServer::CreateTable {
                    name: trimmed_name.to_string(),
                    game_variant: self.table_game_variant,
                    ante,
                    limit_small,
                    limit_big,
                    max_raises,
                };

                // Connect to embedded server instead of central server
                let local_url = format!("ws://127.0.0.1:{}/ws", self.local_server_port);
                self.url = local_url.clone();
                self.room = trimmed_name.to_string();
                self.pending_table_creation = Some(create_table_cmd);

                // Force a reconnection by resetting connection state
                self.tx_out = None;
                self.connected = false;
                self.connecting = true;
                self.app_state = AppState::ConnectOverlay; // Show connecting state while connecting to embedded server
                self.log(format!("🏠 Creating table on your local server at {}", local_url));
            }

            Msg::EmbeddedServerStarted(port) => {
                self.local_server_port = port;

                // Create and start the embedded server directly
                let server = crate::embedded_server::EmbeddedServer::new(port);

                // For simplicity, we'll start it in a background task and store it immediately
                let server_clone = server.clone();
                let server_handle = tokio::spawn(async move {
                    if let Err(e) = server_clone.start().await {
                        eprintln!("Failed to start embedded server: {}", e);
                    }
                });

                // Create a new server instance for storage (without the handle complexity)
                self.embedded_server = Some(crate::embedded_server::EmbeddedServer::new(port));
                self.log(format!("✅ Embedded server starting on port {}", port));

                if self.is_hosting {
                    // If hosting, set host info and go directly to dealer selection
                    self.host_name = Some(self.name.clone());
                    self.host_server_port = Some(port);

                    // Announce hosting to other clients via temporary file
                    let host_info = format!("{}:{}", self.name, port);
                    if let Err(e) = std::fs::write("/tmp/cctmog_host", host_info) {
                        self.log(format!("Warning: Could not announce hosting: {}", e));
                    } else {
                        self.log(format!("📡 Announced hosting on port {}", port));
                    }

                    self.app_state = crate::states::AppState::DealerSelection;
                    self.log("🎯 Server started! Now select a dealer.".to_string());
                    return Task::none();
                } else {
                    // Give it a moment to start, then proceed with table creation
                    return Task::perform(
                        async {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        },
                        |_| Msg::StartEmbeddedServerForTable,
                    );
                }
            }

            Msg::EmbeddedServerError(err) => {
                self.table_creation_error = Some(format!("Failed to start embedded server: {}", err));
                self.log(format!("❌ Embedded server error: {}", err));
            }

            Msg::ToggleAssetTest => self.show_asset_test = !self.show_asset_test,

            // Comments phase messages
            Msg::CommentInputChanged(input) => {
                self.comment_input = input;
            }
            Msg::PostComment => {
                if !self.comment_input.trim().is_empty() {
                    self.send(ClientToServer::PostComment {
                        message: self.comment_input.clone(),
                    });
                    self.comment_input.clear();
                }
            }
            Msg::ContinueToNextGame => {
                self.ready_to_continue = true;
                self.send(ClientToServer::ContinueToNextGame);
            }

            // Lounge menu handlers
            Msg::ViewStats => {
                // TODO: Implement statistics view
                self.log("📊 Statistics view not yet implemented".to_string());
            }
            Msg::OpenSettings => {
                // TODO: Implement settings view
                self.log("⚙️ Settings view not yet implemented".to_string());
            }
            Msg::OpenTutorial => {
                // TODO: Implement tutorial view
                self.log("📖 Tutorial view not yet implemented".to_string());
            }

            // Window events
            Msg::WindowResized(size) => {
                self.window_size = Some(size);
            }

            // Dealer selection messages
            Msg::DealerSelected(dealer_name) => {
                self.selected_dealer = Some(dealer_name);
                self.app_state = AppState::DealerSplash;
                self.dealer_splash_start_time = Some(std::time::Instant::now());
            }

            Msg::DealerSplashFinished => {
                self.app_state = AppState::GameSelection;
                self.dealer_splash_start_time = None;
            }


            // Dealer screen navigation
            Msg::GoToDealerSelection => {
                self.app_state = AppState::DealerSelection;
            }

            // Host game message - delegate to state handler
            Msg::HostGame => {
                if self.app_state == AppState::Lounge {
                    return self.handle_lounge_msg(&msg);
                }
            }

            // Host discovery
            Msg::CheckForHost => {
                if !self.is_hosting {
                    self.check_for_available_host();
                }
            }

            // Host game controls - delegate to game state if in game
            Msg::StartGameNow | Msg::WaitForMorePlayers => {
                if self.app_state == AppState::Game {
                    return self.handle_game_msg(&msg);
                } else {
                    self.log("⚠️ Game controls only available in game".to_string());
                }
            }

            // Game variant selection - connect to server for hosted games
            Msg::GameVariantChosen(variant) => {
                self.log(format!("🎮 Game variant selected: {:?}", variant));

                if self.is_hosting {
                    // If hosting, send the game variant to server and join the room
                    self.send(cctmog_protocol::ClientToServer::SelectGameVariant { variant });

                    // Set connection details for local server
                    self.url = format!("ws://127.0.0.1:{}/ws", self.local_server_port);
                    self.room = "shared_game_room".to_string();
                    self.app_state = AppState::ConnectOverlay;
                    self.connecting = true;
                } else {
                    // If not hosting, just send to existing server
                    self.send(cctmog_protocol::ClientToServer::SelectGameVariant { variant });
                    self.app_state = AppState::Game;
                }
            }
        }
        Task::none()
    }

    fn send(&mut self, cmd: ClientToServer) {
        println!("📤 Attempting to send: {:?}", cmd);
        if let Some(tx) = &self.tx_out {
            let json = serde_json::to_string(&cmd).ok();
            match tx.unbounded_send(cmd) {
                Ok(_) => {
                    println!("✅ Successfully sent message to WebSocket");
                    if let Some(js) = json {
                        self.log(format!("sent: {js}"));
                    }
                }
                Err(e) => {
                    println!("❌ Failed to send message: {:?}", e);
                    self.log(format!("send error: {:?}", e));
                }
            }
        } else {
            println!("❌ No WebSocket connection available");
            self.log("not connected");
        }
    }

    pub fn subscription(&self) -> Subscription<Msg> {
        let tick = iced::time::every(Duration::from_millis(400)).map(|_| Msg::Tick);
        let ws_sub = if (self.app_state == AppState::ConnectOverlay || self.app_state == AppState::Game || self.app_state == AppState::Comments) && (self.connecting || self.connected) && !self.name.trim().is_empty() {
            subscription(self.url.clone(), self.room.clone(), self.name.clone())
        } else {
            Subscription::none()
        };
        let window_sub = iced::window::resize_events().map(|(_, size)| Msg::WindowResized(size));
        Subscription::batch(vec![tick, ws_sub, window_sub])
    }

    pub(crate) fn view(&self) -> Element<Msg> {
        use iced::{Alignment::*, Length::*};
        use iced_widget::column;

        let main_content = match self.app_state {
            AppState::Splash => splash_view(),
            AppState::NameInput => name_input_view(&self.name, &self.name_error),
            AppState::Lounge => self.lounge_view(),
            AppState::TableChoice => table_choice_view(self),
            AppState::TableCreation => table_creation_view(self),
            AppState::TableBrowser => table_browser_view(self),
            AppState::ConnectOverlay => connect_overlay(&self.url, &self.name, &self.room),
            AppState::Game => self.game_view_impl(),
            AppState::Comments => comments_view(self),
            AppState::DealerSelection => self.dealer_selection_view(),
            AppState::DealerSplash => self.dealer_splash_view(),
            AppState::GameSelection => self.game_selection_view(),
        };

        // Only show footer if not in splash screens
        if self.app_state == AppState::Splash || self.app_state == AppState::DealerSplash {
            main_content
        } else {
            column![
                iced_widget::container(main_content).height(Length::Fill),
                footer(self, self.window_size)
            ]
            .height(Length::Fill)
            .into()
        }
    }

    // This was the splash_view but got incorrectly renamed - removing it

    pub fn name_input_view_impl(&self) -> Element<Msg> {
        use iced::{Alignment::*, Length::*};

        container(
            column![
                Space::with_height(Length::Fixed(60.0)),
                container(brand_logo())
                    .center_x(Fill),
                Space::with_height(Length::Fixed(60.0)),
                container(
                    text::<iced::Theme, iced::Renderer>("What's your name?")
                        .size(32)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                            ..Default::default()
                        })
                )
                .center_x(Fill),
                Space::with_height(Length::Fixed(40.0)),
                container(
                    column![
                        text_input("Enter your name", &self.name)
                            .on_input(Msg::NameChanged)
                            .on_submit(Msg::ConfirmName)
                            .padding(15)
                            .size(18)
                            .width(Length::Fixed(300.0))
                            .style(|_theme: &iced::Theme, _status| iced_widget::text_input::Style {
                                background: iced::Background::Color(iced::Color::from_rgb(0.12, 0.12, 0.14)),
                                border: iced::Border {
                                    color: iced::Color::from_rgb(0.25, 0.25, 0.25),
                                    width: 1.0,
                                    radius: iced::border::Radius::from(8.0),
                                },
                                icon: iced::Color::from_rgb(0.7, 0.7, 0.7),
                                placeholder: iced::Color::from_rgb(0.6, 0.6, 0.6),
                                value: iced::Color::from_rgb(0.92, 0.92, 0.94),
                                selection: iced::Color::from_rgb(0.36, 0.62, 0.98),
                            }),
                        Space::with_height(Length::Fixed(10.0)),
                        // Error message display
                        if let Some(ref error) = self.name_error {
                            Element::from(container(
                                text(error)
                                    .size(14)
                                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                        color: Some(iced::Color::from_rgb(0.93, 0.4, 0.4)),
                                        ..Default::default()
                                    })
                            )
                            .center_x(Fill))
                        } else {
                            Element::from(Space::with_height(Length::Fixed(0.0)))
                        },
                        Space::with_height(Length::Fixed(20.0)),
                        // Continue button - conditionally enabled
                        {
                            let trimmed = self.name.trim();
                            let is_valid = !trimmed.is_empty()
                                && trimmed.len() >= 2
                                && trimmed.len() <= 20
                                && trimmed.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-');

                            let mut btn = button(
                                text::<iced::Theme, iced::Renderer>("Continue")
                                    .size(20)
                                    .style(move |_theme| iced_widget::text::Style {
                                        color: Some(if is_valid {
                                            iced::Color::from_rgb(0.92, 0.92, 0.94)
                                        } else {
                                            iced::Color::from_rgb(0.6, 0.6, 0.6)
                                        }),
                                        ..Default::default()
                                    })
                            )
                            .padding(15)
                            .width(Length::Fixed(300.0))
                            .style(move |_theme: &iced::Theme, _status| iced_widget::button::Style {
                                background: Some(iced::Background::Color(if is_valid {
                                    iced::Color::from_rgb(0.36, 0.62, 0.98)
                                } else {
                                    iced::Color::from_rgb(0.3, 0.3, 0.3)
                                })),
                                border: iced::Border {
                                    color: if is_valid {
                                        iced::Color::from_rgb(0.46, 0.72, 1.0)
                                    } else {
                                        iced::Color::from_rgb(0.4, 0.4, 0.4)
                                    },
                                    width: 2.0,
                                    radius: iced::border::Radius::from(8.0),
                                },
                                ..Default::default()
                            });

                            if is_valid {
                                btn = btn.on_press(Msg::ConfirmName);
                            }

                            btn
                        }
                    ]
                    .spacing(10.0)
                    .align_x(Center)
                )
                .center_x(Fill),
            ]
            .align_x(Center)
        )
        .center_x(Fill)
        .center_y(Fill)
        .width(Fill)
        .height(Fill)
        .style(|_theme: &iced::Theme| iced_widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.10, 0.10, 0.11))),
            ..Default::default()
        })
        .into()
    }

    pub fn table_choice_view_impl(&self) -> Element<Msg> {
        use iced::{Alignment::*, Length::*};

        container(
            column![
                Space::with_height(Length::Fixed(60.0)),
                container(brand_logo())
                    .center_x(Fill),
                Space::with_height(Length::Fixed(60.0)),
                container(
                    text::<iced::Theme, iced::Renderer>("Choose an Option")
                        .size(32)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                            ..Default::default()
                        })
                )
                .center_x(Fill),
                Space::with_height(Length::Fixed(40.0)),
                row![
                    container(
                        button(
                            column![
                                text::<iced::Theme, iced::Renderer>("Create")
                                    .size(24)
                                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                        color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                                        ..Default::default()
                                    }),
                                Space::with_height(Length::Fixed(10.0)),
                                text::<iced::Theme, iced::Renderer>("Start a new game room")
                                    .size(16)
                                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                        color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                        ..Default::default()
                                    }),
                            ]
                            .align_x(Center)
                            .spacing(5)
                        )
                        .on_press(Msg::CreateTable)
                        .padding(40)
                        .style(|_theme: &iced::Theme, _status| button::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(0.14, 0.14, 0.16))),
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.36, 0.62, 0.98),
                                width: 2.0,
                                radius: iced::border::Radius::from(10.0),
                            },
                            ..Default::default()
                        })
                    )
                    .width(Length::Fixed(300.0))
                    .height(Length::Fixed(200.0)),
                    Space::with_width(Length::Fixed(40.0)),
                    container(
                        button(
                            column![
                                text::<iced::Theme, iced::Renderer>("Join")
                                    .size(24)
                                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                        color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                                        ..Default::default()
                                    }),
                                Space::with_height(Length::Fixed(10.0)),
                                text::<iced::Theme, iced::Renderer>("Browse available games")
                                    .size(16)
                                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                        color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                        ..Default::default()
                                    }),
                            ]
                            .align_x(Center)
                            .spacing(5)
                        )
                        .on_press(Msg::JoinTable)
                        .padding(40)
                        .style(|_theme: &iced::Theme, _status| button::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(0.14, 0.14, 0.16))),
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.36, 0.62, 0.98),
                                width: 2.0,
                                radius: iced::border::Radius::from(10.0),
                            },
                            ..Default::default()
                        })
                    )
                    .width(Length::Fixed(300.0))
                    .height(Length::Fixed(200.0)),
                ]
                .align_y(Center),
                Space::with_height(Fill),
            ]
            .align_x(Center)
        )
        .center_x(Fill)
        .center_y(Fill)
        .width(Fill)
        .height(Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.10, 0.10, 0.11))),
            ..Default::default()
        })
        .into()
    }

    pub fn table_browser_view_impl(&self) -> Element<Msg> {
        use iced::{Alignment::*, Length::*};

        container(
            column![
                Space::with_height(Length::Fixed(40.0)),
                container(
                    text::<iced::Theme, iced::Renderer>("Available Tables")
                        .size(32)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                            ..Default::default()
                        })
                )
                .center_x(Fill),
                if self.available_tables.is_empty() {
                    container(
                        text::<iced::Theme, iced::Renderer>("No tables available")
                            .size(18)
                            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                ..Default::default()
                            })
                    )
                    .center_x(Fill)
                } else {
                    container(
                        column(
                            self.available_tables.iter().map(|table| {
                                let server_info = if let Some(port) = table.server_port {
                                    format!("🏠 Distributed (port {})", port)
                                } else {
                                    "🌐 Central Server".to_string()
                                };
                                let info_text = format!("Players: {} | Phase: {:?} | {}", table.player_count, table.phase, server_info);
                                button(
                                    column![
                                        text(&table.name)
                                            .size(18),
                                        text(info_text)
                                            .size(14),
                                    ]
                                    .spacing(4.0)
                                )
                                .on_press(Msg::JoinTableByName(table.name.clone()))
                                .padding(15)
                                .into()
                            }).collect::<Vec<_>>()
                        )
                        .spacing(10.0)
                        .align_x(Center)
                    )
                    .center_x(Fill)
                },
                Space::with_height(Length::Fixed(40.0)),
                container(
                    button(
                        text::<iced::Theme, iced::Renderer>("Create")
                            .size(20)
                            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                                ..Default::default()
                            })
                    )
                    .on_press(Msg::CreateNewGame)
                    .padding(20)
                    .style(|_theme: &iced::Theme, _status| button::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.20, 0.60, 0.20))),
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.30, 0.70, 0.30),
                            width: 2.0,
                            radius: iced::border::Radius::from(8.0),
                        },
                        ..Default::default()
                    })
                )
                .center_x(Fill),
                Space::with_height(Fill),
                container(
                    button(
                        text("Back to Home")
                            .size(20)
                            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                                ..Default::default()
                            })
                    )
                    .on_press(Msg::BackToHome)
                    .padding(20)
                    .style(|_theme: &iced::Theme, _status| button::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.14, 0.14, 0.16))),
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.36, 0.62, 0.98),
                            width: 2.0,
                            radius: iced::border::Radius::from(8.0),
                        },
                        ..Default::default()
                    })
                )
                .center_x(Fill),
                Space::with_height(Length::Fixed(40.0)),
            ]
            .align_x(Center)
        )
        .center_x(Fill)
        .center_y(Fill)
        .width(Fill)
        .height(Fill)
        .style(|_theme: &iced::Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.10, 0.10, 0.11))),
            ..Default::default()
        })
        .into()
    }

    pub fn game_view_impl(&self) -> Element<Msg> {
        use iced::{Alignment::*, Length::*};

        // Handle connecting/loading state
        if self.connecting {
            return container(
                text::<iced::Theme, iced::Renderer>("Connecting…")
                    .size(24),
            )
            .center_x(Fill)
            .center_y(Fill)
            .into();
        }

        if self.snapshot.is_none() {
            return container(
                text::<iced::Theme, iced::Renderer>("Waiting for game data...")
                    .size(24),
            )
            .center_x(Fill)
            .center_y(Fill)
            .into();
        }

        let s = self.snapshot.as_ref().unwrap();

        let header = row![
            brand_logo(),
            Space::with_width(12.0),
            row![
                crate::ui::pill(format!("Room {}", s.room)),
                Space::with_width(8.0),
                crate::ui::pill(format!(
                    "Phase: {:?}{}",
                    s.phase,
                    if s.in_betting { " · Betting" } else { " · Draw" }
                )),
                Space::with_width(8.0),
                crate::ui::pill(format!("Round {}", s.round)),
            ]
            .spacing(8.0)
            .align_y(Center),
            Space::with_width(Fill),
            crate::ui::pill(format!("Pot {}", s.pot)),
        ]
            .align_y(Center);

        let seats_ring = round_table_view(s, self.your_id, self.your_seat, &self.your_hand);

        // Your face-up cards (above hole cards)
        let your_up: Element<Msg> = if let Some(me) = s.players.iter().find(|p| {
            self.your_id.map(|id| p.id == id).unwrap_or(false)
                || self.your_seat.map(|seat| p.seat == seat).unwrap_or(false)
        }) {
            if !me.up_cards.is_empty() {
                container(
                    row![
                        text::<iced::Theme, iced::Renderer>("Up:").size(14),
                        Space::with_width(6.0),
                        cards_row_svg(&me.up_cards, CardSize::Small, 6.0),
                    ]
                        .spacing(6.0)
                        .align_y(Alignment::Center),
                )
                    .width(Fill)
                    .center_x(Fill)
                    .into()
            } else {
                Space::with_height(0.0).into()
            }
        } else {
            Space::with_height(0.0).into()
        };

        // Your hole cards (below felt)
        let your_down: Element<Msg> =
            if s.phase != Phase::Lobby && !self.your_hand.down_cards.is_empty() {
                container(
                    row![cards_row_svg(&self.your_hand.down_cards, CardSize::Large, 10.0)]
                        .spacing(10.0)
                        .align_y(Alignment::Center),
                )
                    .width(Fill)
                    .center_x(Fill)
                    .padding([6_u16, 0_u16])
                    .into()
            } else {
                Space::with_height(0.0).into()
            };

        let actions = render_action_bar(s, self.your_seat, self.in_turn(s));

        // Scheduling panel
        let scheduling_panel: Element<Msg> = if s.phase == Phase::Lobby {
            container(
                column![
                    text::<iced::Theme, iced::Renderer>("Scheduling").size(16)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                            ..Default::default()
                        }),
                    horizontal_rule(1),
                    // Show scheduled time if set
                    if let Some(ref scheduled_time) = s.scheduled_start {
                        column![
                            text::<iced::Theme, iced::Renderer>(format!("Scheduled: {}", scheduled_time)).size(12)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                                    ..Default::default()
                                }),
                            text::<iced::Theme, iced::Renderer>(format!("Check-ins: {}/{}", s.checked_in_players.len(), s.players.len())).size(12)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.6, 0.9, 0.6)),
                                    ..Default::default()
                                }),
                            Space::with_height(4.0),
                            // Check-in button if game is scheduled and player hasn't checked in
                            if let Some(your_id) = self.your_id {
                                if !s.checked_in_players.contains(&your_id) {
                                    button(text::<iced::Theme, iced::Renderer>("Check In").size(12))
                                        .on_press(Msg::CheckIn)
                                        .padding([6_u16, 10_u16])
                                        .style(|_theme: &iced::Theme, _status| button::Style {
                                            background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.7, 0.2))),
                                            text_color: iced::Color::WHITE,
                                            border: iced::Border {
                                                color: iced::Color::from_rgb(0.1, 0.6, 0.1),
                                                width: 1.0,
                                                radius: iced::border::Radius::from(4.0),
                                            },
                                            ..Default::default()
                                        })
                                        .into()
                                } else {
                                    text::<iced::Theme, iced::Renderer>("✓ Checked In").size(12)
                                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                            color: Some(iced::Color::from_rgb(0.2, 0.8, 0.2)),
                                            ..Default::default()
                                        })
                                        .into()
                                }
                            } else {
                                Element::from(Space::with_height(0.0))
                            }
                        ]
                    } else {
                        // Schedule a game input
                        column![
                            text::<iced::Theme, iced::Renderer>("Schedule a game:").size(12)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                                    ..Default::default()
                                }),
                            Space::with_height(4.0),
                            row![
                                text_input("e.g. 2024-01-01T20:00", &self.schedule_time_input)
                                    .on_input(Msg::ScheduleTimeChanged)
                                    .on_submit(Msg::ScheduleGame)
                                    .padding(4)
                                    .size(10)
                                    .style(|_theme: &iced::Theme, _status| text_input::Style {
                                        background: iced::Background::Color(iced::Color::from_rgb(0.12, 0.12, 0.14)),
                                        border: iced::Border {
                                            color: iced::Color::from_rgb(0.36, 0.62, 0.98),
                                            width: 1.0,
                                            radius: iced::border::Radius::from(4.0),
                                        },
                                        icon: iced::Color::from_rgb(0.7, 0.7, 0.7),
                                        placeholder: iced::Color::from_rgb(0.5, 0.5, 0.5),
                                        value: iced::Color::from_rgb(0.92, 0.92, 0.94),
                                        selection: iced::Color::from_rgb(0.36, 0.62, 0.98),
                                    }),
                                Space::with_width(4.0),
                                button(text::<iced::Theme, iced::Renderer>("Set").size(10))
                                    .on_press(Msg::ScheduleGame)
                                    .padding([4_u16, 8_u16])
                                    .style(|_theme: &iced::Theme, _status| button::Style {
                                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.36, 0.62, 0.98))),
                                        text_color: iced::Color::WHITE,
                                        border: iced::Border {
                                            color: iced::Color::from_rgb(0.30, 0.56, 0.92),
                                            width: 1.0,
                                            radius: iced::border::Radius::from(4.0),
                                        },
                                        ..Default::default()
                                    })
                            ].spacing(2.0)
                        ].into()
                    }
                ]
                .spacing(4.0),
            )
            .padding(8.0)
            .width(Length::Fill)
            .into()
        } else {
            Space::with_height(0.0).into()
        };

        // Dealer selection panel
        let dealer_panel: Element<Msg> = if s.phase == Phase::Lobby {
            // Check if the player is in the room to allow dealer selection
            let can_select_dealer = self.your_id.is_some() &&
                s.players.iter().any(|p| self.your_id.map(|id| p.id == id).unwrap_or(false));

            container(
                column![
                    text::<iced::Theme, iced::Renderer>("Game Setup").size(16)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                            ..Default::default()
                        }),
                    horizontal_rule(1),
                    text::<iced::Theme, iced::Renderer>(format!("Current: {}", s.game_variant)).size(12)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                            ..Default::default()
                        }),
                    Space::with_height(8.0),
                    if can_select_dealer {
                        button(
                            text::<iced::Theme, iced::Renderer>("🎯 Select Dealer & Game")
                                .size(14)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                                    ..Default::default()
                                })
                        )
                        .on_press(Msg::GoToDealerSelection)
                        .padding([12_u16, 16_u16])
                        .width(Length::Fill)
                        .style(|_theme: &iced::Theme, status| button::Style {
                            background: Some(iced::Background::Color(match status {
                                iced_widget::button::Status::Hovered => iced::Color::from_rgb(0.46, 0.72, 1.0),
                                _ => iced::Color::from_rgb(0.36, 0.62, 0.98),
                            })),
                            text_color: iced::Color::WHITE,
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.56, 0.82, 1.0),
                                width: 2.0,
                                radius: iced::border::Radius::from(8.0),
                            },
                            ..Default::default()
                        })
                        .into()
                    } else {
                        Element::from(
                            text::<iced::Theme, iced::Renderer>("Only players in the room can select dealer and game").size(10)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                                    ..Default::default()
                                })
                        )
                    }
                ]
                .spacing(4.0),
            )
            .padding(8.0)
            .width(Length::Fill)
            .into()
        } else {
            Space::with_height(0.0).into()
        };

        let toggle_log = button(text::<iced::Theme, iced::Renderer>(if self.show_asset_test { "Hide log" } else { "Show log" }))
            .on_press(Msg::ToggleAssetTest)
            .padding([6_u16, 10_u16]);

        let log_panel: Element<Msg> = if self.show_asset_test {
            container(
                column![
                    text::<iced::Theme, iced::Renderer>("Log").size(16),
                    horizontal_rule(1),
                    text::<iced::Theme, iced::Renderer>(self.log.join("\n")).size(14),
                ]
                    .spacing(6.0),
            )
                .padding(8.0)
                .width(Length::Fill)
                .into()
        } else {
            Space::with_height(0.0).into()
        };

        // Chat panel
        let chat_display_text = if self.chat_messages.is_empty() {
            "No messages yet".to_string()
        } else {
            self.chat_messages.iter()
                .map(|(name, msg, scope)| {
                    let scope_prefix = match scope {
                        MessageScope::Match => "[Match]",
                        MessageScope::Group => "[Group]",
                        MessageScope::Global => "[Global]",
                        MessageScope::Private => "[Private]",
                    };
                    format!("{} {}: {}", scope_prefix, name, msg)
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        let chat_panel: Element<Msg> = container(
            column![
                text::<iced::Theme, iced::Renderer>("Chat").size(16)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                        ..Default::default()
                    }),
                horizontal_rule(1),
                container(
                    text::<iced::Theme, iced::Renderer>(chat_display_text)
                        .size(12)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                            ..Default::default()
                        })
                )
                .height(Length::Fixed(100.0))
                .width(Length::Fill)
                .style(|_theme: &iced::Theme| container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(0.08, 0.08, 0.09))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.25, 0.25, 0.25),
                        width: 1.0,
                        radius: iced::border::Radius::from(4.0),
                    },
                    ..Default::default()
                }),
                row![
                    text_input("Type a message...", &self.chat_input)
                        .on_input(Msg::ChatInputChanged)
                        .on_submit(Msg::SendChat)
                        .padding(8)
                        .size(12)
                        .style(|_theme: &iced::Theme, _status| text_input::Style {
                            background: iced::Background::Color(iced::Color::from_rgb(0.12, 0.12, 0.14)),
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.36, 0.62, 0.98),
                                width: 1.0,
                                radius: iced::border::Radius::from(4.0),
                            },
                            icon: iced::Color::from_rgb(0.7, 0.7, 0.7),
                            placeholder: iced::Color::from_rgb(0.5, 0.5, 0.5),
                            value: iced::Color::from_rgb(0.92, 0.92, 0.94),
                            selection: iced::Color::from_rgb(0.36, 0.62, 0.98),
                        }),
                    Space::with_width(4.0),
                    button(text::<iced::Theme, iced::Renderer>("Send").size(12))
                        .on_press(Msg::SendChat)
                        .padding([6_u16, 10_u16])
                        .style(|_theme: &iced::Theme, _status| button::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(0.36, 0.62, 0.98))),
                            text_color: iced::Color::WHITE,
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.30, 0.56, 0.92),
                                width: 1.0,
                                radius: iced::border::Radius::from(4.0),
                            },
                            ..Default::default()
                        })
                ]
                .spacing(4.0),
            ]
                .spacing(6.0),
        )
            .padding(8.0)
            .width(Length::Fill)
            .into();

        let left = column![seats_ring, your_up, your_down]
            .spacing(8.0)
            .width(Length::FillPortion(3));

        let back_home_btn = button(text::<iced::Theme, iced::Renderer>("Back to Home"))
            .on_press(Msg::BackToHome)
            .padding([6_u16, 10_u16])
            .style(|_theme: &iced::Theme, _status| button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    color: iced::Color::from_rgb(0.4, 0.4, 0.4),
                    width: 1.0,
                    radius: iced::border::Radius::from(4.0),
                },
                ..Default::default()
            });

        let right = column![actions, Space::with_height(6.0), scheduling_panel, Space::with_height(6.0), dealer_panel, Space::with_height(8.0), toggle_log, Space::with_height(6.0), back_home_btn, Space::with_height(6.0), log_panel, chat_panel]
            .spacing(8.0)
            .width(Length::FillPortion(1));

        container(
            column![header, Space::with_height(6.0), row![left, right].spacing(14.0).height(Length::Fill)]
                .spacing(8.0)
                .padding(12.0),
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub(crate) fn in_turn(&self, s: &PublicRoom) -> bool {
        let my_seat_opt = if let Some(id) = self.your_id {
            s.players.iter().find(|p| p.id == id).map(|p| p.seat)
        } else {
            self.your_seat
        };

        if let Some(my_seat) = my_seat_opt {
            let me = s.players.iter().find(|p| p.seat == my_seat);
            s.phase == Phase::Acting
                && s.to_act_seat == my_seat
                && me.map(|p| !p.folded && !p.standing).unwrap_or(false)
        } else {
            false
        }
    }
}

