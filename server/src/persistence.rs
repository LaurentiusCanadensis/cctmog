use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use zeromq::{Socket, SocketRecv, SocketSend, ZmqMessage};
use cctmog_protocol::{PublicRoom, GameVariant, Phase};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    RoomCreated {
        room_id: String,
        variant: GameVariant,
        timestamp: DateTime<Utc>,
    },
    PlayerJoined {
        room_id: String,
        player_id: Uuid,
        player_name: String,
        seat: usize,
        timestamp: DateTime<Utc>,
    },
    PlayerLeft {
        room_id: String,
        player_id: Uuid,
        timestamp: DateTime<Utc>,
    },
    GameStarted {
        room_id: String,
        dealer_seat: usize,
        timestamp: DateTime<Utc>,
    },
    CardDealt {
        room_id: String,
        player_id: Uuid,
        card_face_up: bool,
        timestamp: DateTime<Utc>,
    },
    PlayerAction {
        room_id: String,
        player_id: Uuid,
        action: String,
        amount: Option<u64>,
        timestamp: DateTime<Utc>,
    },
    PhaseChanged {
        room_id: String,
        from_phase: Phase,
        to_phase: Phase,
        timestamp: DateTime<Utc>,
    },
    GameEnded {
        room_id: String,
        winners: Vec<Uuid>,
        final_pot: u64,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerStats {
    pub player_id: Uuid,
    pub player_name: String,
    pub games_played: u64,
    pub games_won: u64,
    pub total_winnings: i64, // Can be negative
    pub favorite_variant: Option<GameVariant>,
    pub last_played: DateTime<Utc>,
}

pub struct PersistenceManager {
    event_publisher: zeromq::PubSocket,
    stats_requester: zeromq::ReqSocket,
    local_stats: HashMap<Uuid, PlayerStats>,
}

impl PersistenceManager {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Event publisher for game events
        let mut event_publisher = zeromq::PubSocket::new();
        event_publisher.bind("tcp://*:5555").await?;

        // Request-Reply for stats queries
        let mut stats_requester = zeromq::ReqSocket::new();
        stats_requester.connect("tcp://localhost:5556").await?;

        Ok(Self {
            event_publisher,
            stats_requester,
            local_stats: HashMap::new(),
        })
    }

    /// Publish a game event to the persistence layer
    pub async fn publish_event(&mut self, event: GameEvent) -> Result<(), Box<dyn std::error::Error>> {
        let topic = match &event {
            GameEvent::RoomCreated { .. } => "room.created",
            GameEvent::PlayerJoined { .. } => "player.joined",
            GameEvent::PlayerLeft { .. } => "player.left",
            GameEvent::GameStarted { .. } => "game.started",
            GameEvent::CardDealt { .. } => "card.dealt",
            GameEvent::PlayerAction { .. } => "player.action",
            GameEvent::PhaseChanged { .. } => "phase.changed",
            GameEvent::GameEnded { .. } => "game.ended",
        };

        let serialized = serde_json::to_string(&event)?;
        let message = ZmqMessage::from(format!("{} {}", topic, serialized));

        self.event_publisher.send(message).await?;

        // Also update local stats cache if relevant
        match event {
            GameEvent::PlayerJoined { player_id, player_name, .. } => {
                self.local_stats.entry(player_id).or_insert_with(|| PlayerStats {
                    player_id,
                    player_name: player_name.clone(),
                    games_played: 0,
                    games_won: 0,
                    total_winnings: 0,
                    favorite_variant: None,
                    last_played: Utc::now(),
                });
            }
            GameEvent::GameStarted { .. } => {
                // Increment games_played for all players in room
                // This would require room state lookup
            }
            GameEvent::GameEnded { winners, final_pot, .. } => {
                // Update winner stats
                let winnings_per_winner = final_pot / winners.len() as u64;
                for winner_id in winners {
                    if let Some(stats) = self.local_stats.get_mut(&winner_id) {
                        stats.games_won += 1;
                        stats.total_winnings += winnings_per_winner as i64;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Get player statistics
    pub async fn get_player_stats(&mut self, player_id: Uuid) -> Result<Option<PlayerStats>, Box<dyn std::error::Error>> {
        // First check local cache
        if let Some(stats) = self.local_stats.get(&player_id) {
            return Ok(Some(stats.clone()));
        }

        // Request from persistence service
        let request = serde_json::to_string(&player_id)?;
        let message = ZmqMessage::from(request);

        self.stats_requester.send(message).await?;

        let response = self.stats_requester.recv().await?;
        let response_str = String::from_utf8(response.into_vec())?;

        if response_str == "null" {
            Ok(None)
        } else {
            let stats: PlayerStats = serde_json::from_str(&response_str)?;
            self.local_stats.insert(player_id, stats.clone());
            Ok(Some(stats))
        }
    }

    /// Get game history for a room
    pub async fn get_room_history(&mut self, room_id: &str, limit: usize) -> Result<Vec<GameEvent>, Box<dyn std::error::Error>> {
        #[derive(Serialize)]
        struct HistoryRequest {
            room_id: String,
            limit: usize,
        }

        let request = HistoryRequest {
            room_id: room_id.to_string(),
            limit,
        };

        let request_json = serde_json::to_string(&request)?;
        let message = ZmqMessage::from(format!("history {}", request_json));

        self.stats_requester.send(message).await?;

        let response = self.stats_requester.recv().await?;
        let response_str = String::from_utf8(response.into_vec())?;

        let events: Vec<GameEvent> = serde_json::from_str(&response_str)?;
        Ok(events)
    }

    /// Store room snapshot for recovery
    pub async fn store_room_snapshot(&mut self, room: &PublicRoom) -> Result<(), Box<dyn std::error::Error>> {
        let snapshot_event = GameEvent::PhaseChanged {
            room_id: room.room.clone(),
            from_phase: room.phase,
            to_phase: room.phase, // Same phase, just storing state
            timestamp: Utc::now(),
        };

        // In a real implementation, you'd store the full room state
        // For now, just publish a phase change event
        self.publish_event(snapshot_event).await
    }
}

/// Background service that consumes events and stores them persistently
pub async fn run_persistence_service() -> Result<(), Box<dyn std::error::Error>> {
    let mut subscriber = zeromq::SubSocket::new();
    subscriber.connect("tcp://localhost:5555").await?;
    subscriber.subscribe("").await?; // Subscribe to all topics

    let mut replier = zeromq::RepSocket::new();
    replier.bind("tcp://*:5556").await?;

    // In-memory storage for demo (in production, use a real database)
    let mut events: Vec<GameEvent> = Vec::new();
    let mut player_stats: HashMap<Uuid, PlayerStats> = HashMap::new();

    loop {
        tokio::select! {
            // Handle incoming events
            Ok(message) = subscriber.recv() => {
                let msg_str = String::from_utf8_lossy(&message);
                if let Some((topic, payload)) = msg_str.split_once(' ') {
                    if let Ok(event) = serde_json::from_str::<GameEvent>(payload) {
                        println!("📝 Persisting event: {} - {:?}", topic, event);
                        events.push(event.clone());

                        // Update player stats based on event
                        match event {
                            GameEvent::PlayerJoined { player_id, player_name, .. } => {
                                player_stats.entry(player_id).or_insert_with(|| PlayerStats {
                                    player_id,
                                    player_name: player_name.clone(),
                                    games_played: 0,
                                    games_won: 0,
                                    total_winnings: 0,
                                    favorite_variant: None,
                                    last_played: Utc::now(),
                                });
                            }
                            GameEvent::GameEnded { winners, final_pot, .. } => {
                                let winnings_per_winner = final_pot / winners.len().max(1) as u64;
                                for winner_id in winners {
                                    if let Some(stats) = player_stats.get_mut(&winner_id) {
                                        stats.games_won += 1;
                                        stats.total_winnings += winnings_per_winner as i64;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Handle stats requests
            Ok(request) = replier.recv() => {
                let request_str = String::from_utf8_lossy(&request);

                let response = if request_str.starts_with("history ") {
                    // Room history request
                    let json_part = &request_str[8..];
                    if let Ok(req) = serde_json::from_str::<serde_json::Value>(json_part) {
                        if let Some(room_id) = req["room_id"].as_str() {
                            let limit = req["limit"].as_u64().unwrap_or(100) as usize;
                            let room_events: Vec<_> = events.iter()
                                .filter(|e| match e {
                                    GameEvent::RoomCreated { room_id: r, .. } |
                                    GameEvent::PlayerJoined { room_id: r, .. } |
                                    GameEvent::PlayerLeft { room_id: r, .. } |
                                    GameEvent::GameStarted { room_id: r, .. } |
                                    GameEvent::CardDealt { room_id: r, .. } |
                                    GameEvent::PlayerAction { room_id: r, .. } |
                                    GameEvent::PhaseChanged { room_id: r, .. } |
                                    GameEvent::GameEnded { room_id: r, .. } => r == room_id,
                                })
                                .take(limit)
                                .cloned()
                                .collect();
                            serde_json::to_string(&room_events).unwrap_or_else(|_| "[]".to_string())
                        } else {
                            "[]".to_string()
                        }
                    } else {
                        "[]".to_string()
                    }
                } else {
                    // Player stats request
                    if let Ok(player_id) = serde_json::from_str::<Uuid>(&request_str) {
                        if let Some(stats) = player_stats.get(&player_id) {
                            serde_json::to_string(stats).unwrap_or_else(|_| "null".to_string())
                        } else {
                            "null".to_string()
                        }
                    } else {
                        "null".to_string()
                    }
                };

                replier.send(ZmqMessage::from(response)).await?;
            }
        }
    }
}