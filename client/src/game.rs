use cctmog_protocol::*;
use uuid::Uuid;

#[derive(Debug)]
pub struct Room {
    pub name: String,
    pub game_variant: GameVariant,
    pub ante: u64,
    pub limit_small: u64,
    pub limit_big: u64,
    pub max_raises: u32,

    pub deck: Option<Deck>,
    pub players: Vec<PlayerSeat>,
    pub dealer_seat: usize,
    pub to_act_seat: usize,
    pub pot: u64,
    pub phase: Phase,

    // round & sub-phase
    pub round: u32,
    pub in_betting: bool,

    // draw-phase tracking
    pub draw_started_seat: usize,
    pub draw_acted: Vec<bool>,

    // betting-phase tracking
    pub betting_started_seat: usize,
    pub last_aggressor_seat: Option<usize>,
    pub current_bet: u64,
    pub raises_made: u32,
    pub betting_acted: Vec<bool>,

    // community cards and scheduling
    pub community_cards: Vec<Card>,
    pub scheduled_start: Option<String>,
    pub checked_in_players: Vec<Uuid>,

    // Spectator tracking: list of spectators (non-playing observers)
    pub spectators: Vec<Spectator>,

    // Dealer system tracking
    pub elected_players: Vec<Uuid>,
    pub current_dealer_id: Option<Uuid>,
}

#[derive(Debug)]
pub struct Spectator {
    pub id: Uuid,
    pub name: String,
    pub tx: tokio::sync::mpsc::UnboundedSender<ServerToClient>,
}

#[derive(Debug)]
pub struct PlayerSeat {
    pub id: Uuid,
    pub name: String,
    pub chips: u64,
    pub folded: bool,
    pub standing: bool,
    pub up_cards: Vec<Card>,
    pub down_cards: Vec<Card>,
    pub ready: bool,
    pub committed_round: u64,
    pub tx: tokio::sync::mpsc::UnboundedSender<ServerToClient>,
}

impl Room {
    pub fn new(name: String) -> Self {
        Room {
            name: name.clone(),
            game_variant: GameVariant::default(),
            ante: 10,
            limit_small: 10,
            limit_big: 20,
            max_raises: 3,
            deck: None,
            players: vec![],
            dealer_seat: 0,
            to_act_seat: 0,
            pot: 0,
            phase: Phase::Lobby,
            round: 0,
            in_betting: false,
            draw_started_seat: 0,
            draw_acted: vec![],
            betting_started_seat: 0,
            last_aggressor_seat: None,
            current_bet: 0,
            raises_made: 0,
            betting_acted: vec![],
            community_cards: vec![],
            scheduled_start: None,
            checked_in_players: vec![],
            spectators: vec![],
            elected_players: vec![],
            current_dealer_id: None,
        }
    }

    pub fn add_player(&mut self, id: Uuid, name: String, tx: tokio::sync::mpsc::UnboundedSender<ServerToClient>) -> usize {
        let seat = self.players.len();
        self.players.push(PlayerSeat {
            id,
            name,
            chips: 1000,
            folded: false,
            standing: false,
            up_cards: vec![],
            down_cards: vec![],
            ready: false,
            committed_round: 0,
            tx,
        });
        seat
    }

    pub fn public_snapshot(&self) -> PublicRoom {
        PublicRoom {
            room: self.name.clone(),
            game_variant: self.game_variant,
            dealer_seat: self.dealer_seat,
            to_act_seat: self.to_act_seat,
            pot: self.pot,
            ante: self.ante,
            phase: self.phase.clone(),
            in_betting: self.in_betting,
            current_bet: self.current_bet,
            raises_made: self.raises_made,
            max_raises: self.max_raises,
            round: self.round,
            limit_small: self.limit_small,
            limit_big: self.limit_big,
            community_cards: self.community_cards.clone(),
            scheduled_start: self.scheduled_start.clone(),
            checked_in_players: self.checked_in_players.clone(),
            elected_players: self.elected_players.clone(),
            current_dealer_id: self.current_dealer_id,
            available_variants: vec![GameVariant::SevenTwentySeven, GameVariant::Omaha, GameVariant::TexasHoldem],
            players: self
                .players
                .iter()
                .enumerate()
                .map(|(i, p)| PublicPlayer {
                    id: p.id,
                    name: p.name.clone(),
                    seat: i,
                    chips: p.chips,
                    folded: p.folded,
                    standing: p.standing,
                    up_cards: p.up_cards.clone(),
                    cards_count: p.up_cards.len() + p.down_cards.len(),
                    committed_round: p.committed_round,
                    ready: p.ready,
                })
                .collect(),
        }
    }
}

/// Helper functions for game logic
pub fn seat_of(r: &Room, id: Uuid) -> Option<usize> {
    r.players.iter().position(|p| p.id == id)
}

pub fn all_cards(p: &PlayerSeat) -> Vec<Card> {
    let mut v = p.up_cards.clone();
    v.extend(p.down_cards.iter().copied());
    v
}

pub fn alive_seats(r: &Room) -> Vec<(usize, &PlayerSeat)> {
    r.players
        .iter()
        .enumerate()
        .filter(|(_, p)| !p.folded)
        .collect()
}

pub fn next_alive_left_of(r: &Room, from: usize) -> usize {
    let n = r.players.len();
    let mut i = (from + 1) % n;
    while r.players[i].folded {
        i = (i + 1) % n;
    }
    i
}

pub fn bet_size_for_round(r: &Room) -> u64 {
    if r.round <= 2 {
        r.limit_small
    } else {
        r.limit_big
    }
}

pub fn commit(r: &mut Room, seat: usize, amount: u64) {
    if amount == 0 {
        return;
    }
    let p = &mut r.players[seat];
    let pay = amount.min(p.chips);
    p.chips -= pay;
    p.committed_round += pay;
    r.pot += pay;
}

/// Convert internal Room to public PublicRoom for client messages
pub fn public_room(r: &Room) -> PublicRoom {
    PublicRoom {
        room: r.name.clone(),
        game_variant: r.game_variant,
        dealer_seat: r.dealer_seat,
        to_act_seat: r.to_act_seat,
        pot: r.pot,
        ante: r.ante,
        phase: r.phase.clone(),
        in_betting: r.in_betting,
        current_bet: r.current_bet,
        raises_made: r.raises_made,
        max_raises: r.max_raises,
        round: r.round,
        limit_small: r.limit_small,
        limit_big: r.limit_big,
        community_cards: r.community_cards.clone(),
        scheduled_start: r.scheduled_start.clone(),
        checked_in_players: r.checked_in_players.clone(),
        elected_players: r.elected_players.clone(),
        current_dealer_id: r.current_dealer_id,
        available_variants: vec![GameVariant::SevenTwentySeven, GameVariant::Omaha, GameVariant::TexasHoldem],
        players: r
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| PublicPlayer {
                id: p.id,
                name: p.name.clone(),
                seat: i,
                chips: p.chips,
                folded: p.folded,
                standing: p.standing,
                up_cards: p.up_cards.clone(),
                cards_count: p.up_cards.len() + p.down_cards.len(),
                committed_round: p.committed_round,
                ready: p.ready,
            })
            .collect(),
    }
}

/// Game state validation functions
#[allow(dead_code)]
pub fn can_take_card(r: &Room, player_id: Uuid) -> Result<usize, String> {
    if r.phase != Phase::Acting {
        return Err(format!("Wrong phase: {:?}", r.phase));
    }
    if r.in_betting {
        return Err("Currently in betting".to_string());
    }

    let seat = seat_of(r, player_id).ok_or("Player not found")?;

    if r.to_act_seat != seat {
        return Err(format!("Not your turn (to_act={} you={})", r.to_act_seat, seat));
    }
    if r.players[seat].folded {
        return Err("Player folded".to_string());
    }
    if r.players[seat].standing {
        return Err("Player already standing".to_string());
    }

    Ok(seat)
}

#[allow(dead_code)]
pub fn can_bet_or_raise(r: &Room, player_id: Uuid, is_raise: bool) -> Result<usize, String> {
    if !r.in_betting || r.phase != Phase::Acting {
        return Err("Not in betting phase".to_string());
    }

    let seat = seat_of(r, player_id).ok_or("Player not found")?;

    if r.to_act_seat != seat {
        return Err(format!("Not your turn (to_act={} you={})", r.to_act_seat, seat));
    }
    if r.players[seat].folded {
        return Err("Player folded".to_string());
    }

    if r.current_bet == 0 && is_raise {
        return Err("Cannot raise when no bet exists".to_string());
    }
    if r.current_bet > 0 && !is_raise {
        return Err("Must raise when bet exists".to_string());
    }
    if is_raise && r.raises_made >= r.max_raises {
        return Err(format!("Maximum raises ({}) reached", r.max_raises));
    }

    Ok(seat)
}

/// Helper function to find the next dealer after the current one rotates
pub fn next_dealer_left_of(r: &Room, current_dealer_seat: usize) -> Option<Uuid> {
    if r.players.is_empty() {
        return None;
    }
    let next_seat = (current_dealer_seat + 1) % r.players.len();
    Some(r.players[next_seat].id)
}

/// Scoring functions for the game
#[allow(dead_code)]
pub fn calculate_low_score(cards: &[Card]) -> Option<u32> {
    if cards.is_empty() {
        return None;
    }

    let mut total = 0;
    for card in cards {
        total += match card.rank {
            Rank::Ace => 1,
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 12,
            Rank::King => 13,
        };
    }

    if total <= 7 {
        Some(total)
    } else {
        None
    }
}

#[allow(dead_code)]
pub fn calculate_high_score(cards: &[Card]) -> (u32, bool) {
    if cards.is_empty() {
        return (0, true); // Busted with no cards
    }

    let mut total = 0;
    for card in cards {
        total += match card.rank {
            Rank::Ace => 1,
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 12,
            Rank::King => 13,
        };
    }

    (total, total > 27)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_low_score_calculation() {
        let cards = vec![
            Card { rank: Rank::Ace, suit: Suit::Spades, face_up: true },
            Card { rank: Rank::Two, suit: Suit::Hearts, face_up: true },
            Card { rank: Rank::Three, suit: Suit::Clubs, face_up: true },
        ]; // Total = 6

        assert_eq!(calculate_low_score(&cards), Some(6));

        let high_cards = vec![
            Card { rank: Rank::King, suit: Suit::Spades, face_up: true },
            Card { rank: Rank::Queen, suit: Suit::Hearts, face_up: true },
        ]; // Total = 25 (too high for low)

        assert_eq!(calculate_low_score(&high_cards), None);
    }

    #[test]
    fn test_high_score_calculation() {
        let cards = vec![
            Card { rank: Rank::King, suit: Suit::Spades, face_up: true },
            Card { rank: Rank::Queen, suit: Suit::Hearts, face_up: true },
            Card { rank: Rank::Two, suit: Suit::Clubs, face_up: true },
        ]; // Total = 27

        let (score, busted) = calculate_high_score(&cards);
        assert_eq!(score, 27);
        assert!(!busted);

        let bust_cards = vec![
            Card { rank: Rank::King, suit: Suit::Spades, face_up: true },
            Card { rank: Rank::Queen, suit: Suit::Hearts, face_up: true },
            Card { rank: Rank::Jack, suit: Suit::Clubs, face_up: true },
        ]; // Total = 36 (busted)

        let (score, busted) = calculate_high_score(&bust_cards);
        assert_eq!(score, 36);
        assert!(busted);
    }

    #[test]
    fn test_seat_finding() {
        let mut room = Room::new("test".to_string());
        let player_id = Uuid::new_v4();

        // Player not in room
        assert_eq!(seat_of(&room, player_id), None);

        // Add player
        room.players.push(PlayerSeat {
            id: player_id,
            name: "Test".to_string(),
            chips: 1000,
            folded: false,
            standing: false,
            up_cards: vec![],
            down_cards: vec![],
            ready: false,
            committed_round: 0,
            tx: tokio::sync::mpsc::unbounded_channel().0,
        });

        // Player found
        assert_eq!(seat_of(&room, player_id), Some(0));
    }
}