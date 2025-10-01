use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// ---- Message Scopes for Chat ----
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageScope {
    /// Message visible to current room/match only
    Match,
    /// Message visible to all connected players globally
    Global,
    /// Message visible to players in the same game group/server
    Group,
    /// Private message between two specific players
    Private,
}

impl Default for MessageScope {
    fn default() -> Self {
        MessageScope::Match
    }
}

/// ---- Game Variants ----
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameVariant {
    SevenTwentySeven,
    Omaha,
    TexasHoldem,
}

impl fmt::Display for GameVariant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameVariant::SevenTwentySeven => write!(f, "7/27"),
            GameVariant::Omaha => write!(f, "Omaha"),
            GameVariant::TexasHoldem => write!(f, "Texas Hold'em"),
        }
    }
}

impl Default for GameVariant {
    fn default() -> Self {
        GameVariant::SevenTwentySeven
    }
}

/// Game configuration constants for different variants
impl GameVariant {
    /// Number of hole cards dealt to each player
    pub fn hole_cards(&self) -> usize {
        match self {
            GameVariant::SevenTwentySeven => 2, // 2 down cards initially
            GameVariant::Omaha => 4,
            GameVariant::TexasHoldem => 2,
        }
    }

    /// Number of community cards (board cards)
    pub fn community_cards(&self) -> usize {
        match self {
            GameVariant::SevenTwentySeven => 0, // No community cards
            GameVariant::Omaha => 5,
            GameVariant::TexasHoldem => 5,
        }
    }

    /// Maximum number of cards a player can have
    pub fn max_cards_per_player(&self) -> usize {
        match self {
            GameVariant::SevenTwentySeven => 7, // Can draw up to 5 more cards
            GameVariant::Omaha => 4, // Only hole cards
            GameVariant::TexasHoldem => 2, // Only hole cards
        }
    }

    /// Whether this variant uses community cards
    pub fn uses_community_cards(&self) -> bool {
        match self {
            GameVariant::SevenTwentySeven => false,
            GameVariant::Omaha => true,
            GameVariant::TexasHoldem => true,
        }
    }
}

/// ---- Cards ----
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
    pub face_up: bool,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let r = match self.rank {
            Rank::Ace => "A",
            Rank::King => "K",
            Rank::Queen => "Q",
            Rank::Jack => "J",
            Rank::Ten => "10",
            Rank::Nine => "9",
            Rank::Eight => "8",
            Rank::Seven => "7",
            Rank::Six => "6",
            Rank::Five => "5",
            Rank::Four => "4",
            Rank::Three => "3",
            Rank::Two => "2",
        };
        let s = match self.suit {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        };
        write!(f, "{}{}", r, s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn standard_shuffled() -> Self {
        let mut cards = Vec::with_capacity(52);
        for &s in &[Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            for r in [
                Rank::Two,
                Rank::Three,
                Rank::Four,
                Rank::Five,
                Rank::Six,
                Rank::Seven,
                Rank::Eight,
                Rank::Nine,
                Rank::Ten,
                Rank::Jack,
                Rank::Queen,
                Rank::King,
                Rank::Ace,
            ] {
                cards.push(Card {
                    rank: r,
                    suit: s,
                    face_up: false,
                });
            }
        }
        cards.shuffle(&mut thread_rng());
        Deck { cards }
    }
    pub fn draw(&mut self, face_up: bool) -> Option<Card> {
        self.cards.pop().map(|mut c| {
            c.face_up = face_up;
            c
        })
    }
}

/// ---- Scoring ----
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Score {
    pub best_under_7: Option<f32>,
    pub dist_to_7: Option<f32>,
    pub best_under_27: Option<f32>,
    pub dist_to_27: Option<f32>,
    pub bust_27: bool,
}

/// Generic hand evaluation for different poker variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HandRank {
    // For Texas Hold'em and Omaha
    HighCard(Vec<Rank>),
    OnePair(Rank, Vec<Rank>),
    TwoPair(Rank, Rank, Rank),
    ThreeOfAKind(Rank, Vec<Rank>),
    Straight(Rank),
    Flush(Vec<Rank>),
    FullHouse(Rank, Rank),
    FourOfAKind(Rank, Rank),
    StraightFlush(Rank),
    RoyalFlush,
    // For 7/27 variant
    SevenTwentySeven(Score),
}

impl HandRank {
    /// Get the strength ranking for comparison (higher is better)
    pub fn strength(&self) -> u8 {
        match self {
            HandRank::HighCard(_) => 0,
            HandRank::OnePair(_, _) => 1,
            HandRank::TwoPair(_, _, _) => 2,
            HandRank::ThreeOfAKind(_, _) => 3,
            HandRank::Straight(_) => 4,
            HandRank::Flush(_) => 5,
            HandRank::FullHouse(_, _) => 6,
            HandRank::FourOfAKind(_, _) => 7,
            HandRank::StraightFlush(_) => 8,
            HandRank::RoyalFlush => 9,
            HandRank::SevenTwentySeven(_) => 10, // Different scoring system
        }
    }
}

pub fn card_value_nonace(rank: Rank) -> f32 {
    match rank {
        Rank::Jack | Rank::Queen | Rank::King => 0.5,
        Rank::Ace => unreachable!(),
        Rank::Two => 2.0,
        Rank::Three => 3.0,
        Rank::Four => 4.0,
        Rank::Five => 5.0,
        Rank::Six => 6.0,
        Rank::Seven => 7.0,
        Rank::Eight => 8.0,
        Rank::Nine => 9.0,
        Rank::Ten => 10.0,
    }
}

pub fn score_hand(cards: &[Card]) -> Score {
    let mut base = 0.0f32;
    let mut aces = 0usize;
    for c in cards {
        if c.rank == Rank::Ace {
            aces += 1;
        } else {
            base += card_value_nonace(c.rank);
        }
    }
    let best_under = |t: f32| {
        let mut best: Option<f32> = None; // explicit type
        for elevens in 0..=aces {
            let total = base + (aces - elevens) as f32 * 1.0 + elevens as f32 * 11.0;
            if total <= t {
                best = Some(best.unwrap_or(total).max(total));
            }
        }
        best
    };
    let under7 = best_under(7.0);
    let under27 = best_under(27.0);
    Score {
        dist_to_7: under7.map(|v| (7.0 - v).abs()),
        best_under_7: under7,
        dist_to_27: under27.map(|v| (27.0 - v).abs()),
        best_under_27: under27,
        bust_27: under27.is_none(),
    }
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PublicPlayer {
    pub id: uuid::Uuid,
    pub name: String,
    pub seat: usize,
    pub chips: u64,
    pub folded: bool,
    pub standing: bool,
    pub up_cards: Vec<Card>,
    pub cards_count: usize,
    pub committed_round: u64,
    // NEW
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateHand {
    pub down_cards: Vec<Card>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Phase {
    Lobby,
    WaitingForDealer, // New phase: minimum 4 players joined, waiting for all to elect to start
    DealerSelection,  // New phase: players have elected, now choosing/delegating dealer
    GameSelection,    // New phase: dealer is choosing game variant
    Dealing,
    Acting,
    Showdown,
    Comments,         // New phase: post-game comments and feedback
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRoom {
    pub room: String,
    pub game_variant: GameVariant,
    pub dealer_seat: usize,
    pub to_act_seat: usize,
    pub pot: u64,
    pub ante: u64,
    pub phase: Phase,
    pub players: Vec<PublicPlayer>,
    // betting state
    pub in_betting: bool,
    pub current_bet: u64,
    pub raises_made: u32,
    pub max_raises: u32,
    pub round: u32,
    pub limit_small: u64,
    pub limit_big: u64,
    pub community_cards: Vec<Card>,
    pub scheduled_start: Option<String>, // ISO 8601 timestamp
    pub checked_in_players: Vec<Uuid>,
    // Dealer system fields
    pub elected_players: Vec<Uuid>, // Players who have elected to start
    pub current_dealer_id: Option<Uuid>, // Current dealer (if any)
    pub available_variants: Vec<GameVariant>, // Available game variants for dealer to choose
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub game_variant: GameVariant,
    pub player_count: usize,
    pub phase: Phase,
    pub server_port: Option<u16>, // None for central server, Some(port) for distributed tables
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientToServer {
    Join { room: String, name: String },
    Leave,
    SitReady,
    StartHand,
    SelectGameVariant { variant: GameVariant },

    // Draw sub-phase
    TakeCard,
    Stand,
    Fold,

    // Betting sub-phase
    Check,
    Bet,
    Call,
    Raise,

    // Scheduling
    ScheduleGame { start_time: String }, // ISO 8601 timestamp
    CheckIn,

    // Chat with different scopes
    Chat { message: String, scope: MessageScope },

    // Private messaging
    PrivateMessage { recipient: Uuid, message: String },

    // Table listing
    ListTables,

    // Spectator mode - join as observer only
    JoinAsSpectator { room: String, name: String },
    LeaveSpectator,

    // Dealer system
    ElectToStart,
    DelegateDealer { player_id: Uuid },
    ChooseGameVariant { variant: GameVariant },

    // Table creation
    CreateTable {
        name: String,
        game_variant: GameVariant,
        ante: u64,
        limit_small: u64,
        limit_big: u64,
        max_raises: u32,
    },
    // Register a distributed table with the central server
    RegisterTable {
        name: String,
        game_variant: GameVariant,
        ante: u64,
        limit_small: u64,
        limit_big: u64,
        max_raises: u32,
        server_port: u16,
        player_count: usize,
    },

    // Comments phase
    PostComment { message: String },
    ContinueToNextGame,

    // Lounge system
    JoinLounge { name: String },
    LeaveLounge,
    VolunteerToHost { port: u16 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerToClient {
    Hello {
        your_id: Uuid,
    },
    Joined {
        snapshot: PublicRoom,
        your_seat: usize,
        your_hand: PrivateHand,
    },
    UpdateState {
        snapshot: PublicRoom,
    },
    YourHand {
        hand: PrivateHand,
    },
    Error {
        message: String,
    },
    Info {
        message: String,
    },
    Showdown {
        winners7: Vec<Uuid>,
        winners27: Vec<Uuid>,
        payouts: Vec<(Uuid, u64)>,
        reveal: Vec<(Uuid, Vec<Card>)>,
    },
    ChatMessage {
        player_name: String,
        message: String,
        scope: MessageScope,
        room: Option<String>,
        timestamp: String,
        recipient: Option<Uuid>, // For private messages
    },
    TableList {
        tables: Vec<TableInfo>,
    },
    SpectatorJoined {
        snapshot: PublicRoom,
    },
    DealerDelegated {
        dealer_id: Uuid,
        dealer_name: String,
    },
    GameVariantSelected {
        variant: GameVariant,
        selected_by: String,
    },
    GameComment {
        comment: GameComment,
    },

    // Lounge updates
    LoungeUpdate {
        players: Vec<String>,
        available_hosts: Vec<(String, u16)>, // (player_name, port)
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub player_name: String,
    pub message: String,
    pub scope: MessageScope,
    pub room: Option<String>,
    pub timestamp: String,
    pub recipient: Option<Uuid>, // For private messages
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameComment {
    pub player_id: Uuid,
    pub player_name: String,
    pub message: String,
    pub timestamp: String,
}
