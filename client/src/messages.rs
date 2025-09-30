#[derive(Debug, Clone)]
pub enum Msg {
    ServerUrlChanged(String),
    NameChanged(String),
    RoomChanged(String),
    ConnectToggle,
    WsConnected(iced::futures::channel::mpsc::UnboundedSender<cctmog_protocol::ClientToServer>),
    WsEvent(cctmog_protocol::ServerToClient),
    WsError(String),

    SitReady,
    StartHand,
    TakeCard,
    Stand,
    Fold,
    Check,
    Bet,
    Call,
    Raise,

    ToggleAssetTest,
    Tick,

    // New messages for splash and table choice
    SplashFinished,
    CreateTable,
    JoinTable,
    BrowseTables,
    CreateNewGame,
    BackToHome,

    // Chat messages
    ChatInputChanged(String),
    SendChat,

    // Join specific table
    JoinTableByName(String),

    // Name confirmation
    ConfirmName,

    // Scheduling messages
    ScheduleGame,
    ScheduleTimeChanged(String),
    CheckIn,

    // Game variant selection
    SelectGameVariant(cctmog_protocol::GameVariant),

    // Table creation form inputs
    TableNameChanged(String),
    TableGameVariantChanged(cctmog_protocol::GameVariant),
    TableAnteChanged(String),
    TableLimitSmallChanged(String),
    TableLimitBigChanged(String),
    TableMaxRaisesChanged(String),
    SubmitTableCreation,
    StartEmbeddedServerForTable,
    EmbeddedServerStarted(u16),
    EmbeddedServerError(String),

    // Comments phase messages
    CommentInputChanged(String),
    PostComment,
    ContinueToNextGame,

    // Lounge menu options
    ViewStats,
    OpenSettings,
    OpenTutorial,

    // Window events
    WindowResized(iced::Size),

    // Dealer selection messages
    DealerSelected(String), // player name
    DealerSplashFinished,

    // Game selection messages
    GameVariantChosen(cctmog_protocol::GameVariant),

    // Dealer screen navigation
    GoToDealerSelection,

    // Host game message
    HostGame,

    // Host discovery
    CheckForHost,

    // Host game controls
    StartGameNow,
    WaitForMorePlayers,
}
