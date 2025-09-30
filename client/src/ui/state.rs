#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    InGame,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameStatus {
    WaitingForPlayers,
    Betting,
    Comments,
    Cards,
    ShowDown,
    GameOver,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub connection: ConnectionState,
    pub game_status: Option<GameStatus>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            connection: ConnectionState::Disconnected,
            game_status: None,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn connecting() -> Self {
        Self {
            connection: ConnectionState::Connecting,
            game_status: None,
        }
    }

    pub fn in_game(status: GameStatus) -> Self {
        Self {
            connection: ConnectionState::InGame,
            game_status: Some(status),
        }
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.connection, ConnectionState::InGame)
    }

    pub fn is_connecting(&self) -> bool {
        matches!(self.connection, ConnectionState::Connecting)
    }

    pub fn is_disconnected(&self) -> bool {
        matches!(self.connection, ConnectionState::Disconnected)
    }

    pub fn can_take_action(&self) -> bool {
        matches!(self.game_status, Some(GameStatus::Betting))
    }
}