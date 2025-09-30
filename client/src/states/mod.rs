pub mod splash;
pub mod name_input;
pub mod lounge;
pub mod table_choice;
pub mod table_creation;
pub mod table_browser;
pub mod connect_overlay;
pub mod game;
pub mod comments;
pub mod dealer_selection;
pub mod dealer_splash;
pub mod game_selection;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Splash,
    NameInput,
    Lounge,
    TableChoice,
    TableCreation,
    TableBrowser,
    ConnectOverlay,
    Game,
    Comments,
    DealerSelection,
    DealerSplash,
    GameSelection,
}