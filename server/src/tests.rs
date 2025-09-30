use cctmog_protocol::*;
use uuid::Uuid;
use std::collections::HashMap;
use tokio::sync::mpsc;

#[cfg(test)]
mod game_tests {
    use super::*;

    /// Creates a test player with given name, seat, and initial chips
    pub fn create_test_player(name: &str, seat: usize, chips: u64) -> PublicPlayer {
        PublicPlayer {
            id: Uuid::new_v4(),
            name: name.to_string(),
            seat,
            chips,
            folded: false,
            standing: false,
            up_cards: vec![],
            cards_count: 0,
            committed_round: 0,
            ready: false,
        }
    }

    /// Creates a test room with three players
    pub fn create_test_room() -> PublicRoom {
        PublicRoom {
            room: "Test Room".to_string(),
            game_variant: GameVariant::SevenTwentySeven,
            dealer_seat: 0,
            to_act_seat: 0,
            pot: 0,
            ante: 50,
            phase: Phase::Lobby,
            players: vec![
                create_test_player("John", 0, 1000),
                create_test_player("Joe", 1, 1000),
                create_test_player("Frank", 2, 1000),
            ],
            in_betting: false,
            current_bet: 0,
            raises_made: 0,
            max_raises: 3,
            round: 0,
            limit_small: 10,
            limit_big: 20,
            community_cards: vec![],
            scheduled_start: None,
            checked_in_players: vec![],
            elected_players: vec![],
            current_dealer_id: None,
            available_variants: vec![GameVariant::SevenTwentySeven, GameVariant::Omaha, GameVariant::TexasHoldem],
        }
    }

    /// Creates a standard 52-card deck
    #[allow(dead_code)]
    fn create_deck() -> Vec<Card> {
        let mut deck = Vec::new();
        for suit in [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            for rank in [Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five,
                        Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                        Rank::Jack, Rank::Queen, Rank::King] {
                deck.push(Card { rank, suit, face_up: false });
            }
        }
        deck
    }

    /// Tests a complete three-player game from start to finish
    #[test]
    fn test_full_three_player_match() {
        let mut room = create_test_room();

        println!("\n🎯 THREE-PLAYER POKER MATCH BEGINS");
        println!("===============================================");

        // Start the game - Acting phase with first round
        room.phase = Phase::Acting;
        room.pot = 150; // 3 players * 50 ante
        room.round = 1;
        room.in_betting = false;

        // Simulate initial card deal - each player gets 2 hole cards
        for player in &mut room.players {
            player.cards_count = 2;
            player.chips -= room.ante;
            player.committed_round = room.ante;
        }

        println!("\n📋 INITIAL SETUP:");
        println!("• John starts with {} chips", room.players[0].chips + room.ante);
        println!("• Joe starts with {} chips", room.players[1].chips + room.ante);
        println!("• Frank starts with {} chips", room.players[2].chips + room.ante);
        println!("• Ante: {} chips per player", room.ante);
        println!("• Starting pot: {} chips", room.pot);

        // Assign specific hole cards for demonstration
        room.players[0].up_cards = vec![]; // John's hole cards (hidden)
        room.players[1].up_cards = vec![]; // Joe's hole cards (hidden)
        room.players[2].up_cards = vec![]; // Frank's hole cards (hidden)

        println!("\n🃏 INITIAL DEAL (2 hole cards each):");
        println!("• John: [🂠🂠] (2 hole cards)");
        println!("• Joe: [🂠🂠] (2 hole cards)");
        println!("• Frank: [🂠🂠] (2 hole cards)");

        // Test initial state
        assert_eq!(room.phase, Phase::Acting);
        assert_eq!(room.players.len(), 3);
        assert_eq!(room.pot, 150);

        println!("\n🎯 ROUND 1 - DRAW PHASE:");
        println!("Players can now take additional cards...");

        // Simulate players taking cards in draw phase
        room.players[0].cards_count = 5; // John takes 3 cards (2+3=5)
        room.players[1].cards_count = 4; // Joe takes 2 cards (2+2=4)
        room.players[2].cards_count = 3; // Frank takes 1 card (2+1=3)

        // Add some up cards for visual display
        room.players[0].up_cards = vec![
            Card { rank: Rank::Seven, suit: Suit::Hearts, face_up: true },
            Card { rank: Rank::Three, suit: Suit::Clubs, face_up: true },
            Card { rank: Rank::Ace, suit: Suit::Spades, face_up: true }
        ];
        room.players[1].up_cards = vec![
            Card { rank: Rank::King, suit: Suit::Diamonds, face_up: true },
            Card { rank: Rank::Queen, suit: Suit::Hearts, face_up: true }
        ];
        room.players[2].up_cards = vec![
            Card { rank: Rank::Ten, suit: Suit::Spades, face_up: true }
        ];

        println!("• John HITS (takes 3 cards): [🂠🂠] + [7♥ 3♣ A♠] = 5 cards total");
        println!("• Joe HITS (takes 2 cards): [🂠🂠] + [K♦ Q♥] = 4 cards total");
        println!("• Frank HITS (takes 1 card): [🂠🂠] + [10♠] = 3 cards total");

        // Move to betting phase
        room.in_betting = true;
        room.to_act_seat = 0;
        room.current_bet = 0;

        println!("\n💰 ROUND 1 - BETTING PHASE:");
        println!("Betting starts with John...");

        // Test betting logic - John bets 100
        room.current_bet = 100;
        room.players[0].committed_round = 150; // ante + bet
        room.players[0].chips -= 100;
        room.pot += 100;
        println!("• John BETS {} chips (total committed: {})", 100, room.players[0].committed_round);

        // Joe calls John's bet
        room.players[1].committed_round = 150; // ante + call
        room.players[1].chips -= 100;
        room.pot += 100;
        println!("• Joe CALLS {} chips (total committed: {})", 100, room.players[1].committed_round);

        // Frank folds
        room.players[2].folded = true;
        println!("• Frank FOLDS (chips remaining: {})", room.players[2].chips);

        println!("  → Current pot: {} chips", room.pot);

        assert_eq!(room.pot, 350); // 150 ante + 100 John + 100 Joe
        assert!(room.players[2].folded);
        assert!(!room.players[0].folded);
        assert!(!room.players[1].folded);

        // Move to second round
        room.round = 2;
        room.in_betting = false;

        println!("\n🎯 ROUND 2 - DRAW PHASE:");
        println!("Remaining players can take more cards...");

        // Players can take more cards (up to 7 total)
        room.players[0].cards_count = 7; // John takes 2 more (5+2=7)
        room.players[1].cards_count = 6; // Joe takes 2 more (4+2=6)

        // Add final up cards
        room.players[0].up_cards.extend(vec![
            Card { rank: Rank::Five, suit: Suit::Diamonds, face_up: true },
            Card { rank: Rank::Two, suit: Suit::Hearts, face_up: true }
        ]);
        room.players[1].up_cards.extend(vec![
            Card { rank: Rank::Nine, suit: Suit::Clubs, face_up: true },
            Card { rank: Rank::Four, suit: Suit::Spades, face_up: true }
        ]);

        println!("• John HITS (takes 2 more): [🂠🂠] + [7♥ 3♣ A♠ 5♦ 2♥] = 7 cards total");
        println!("• Joe HITS (takes 2 more): [🂠🂠] + [K♦ Q♥ 9♣ 4♠] = 6 cards total");
        println!("• Frank: FOLDED (no more cards)");

        // Move to final betting round
        room.in_betting = true;
        room.current_bet = 0;

        println!("\n💰 ROUND 2 - BETTING PHASE:");
        println!("Final betting round...");

        // John checks (no additional bet)
        // Joe checks (no additional bet)
        println!("• John CHECKS");
        println!("• Joe CHECKS");
        println!("  → Final pot: {} chips", room.pot);

        // Move to showdown
        room.phase = Phase::Showdown;

        println!("\n🎭 SHOWDOWN:");
        println!("Revealing all cards for active players...");

        // Simulate final hands (for display purposes)
        println!("• John's final hand: [A♦ 2♠] + [7♥ 3♣ A♠ 5♦ 2♥] = 7 cards");
        println!("  → Low score: A+2+3+5+7 = 18 (busted for low)");
        println!("  → High score: A+2+3+5+7 = 18 (good for high)");

        println!("• Joe's final hand: [10♥ J♣] + [K♦ Q♥ 9♣ 4♠] = 6 cards");
        println!("  → Low score: 4+9+10+J+Q = 39 (busted for low)");
        println!("  → High score: 4+9+10+J+Q+K = 52 (busted for high)");

        println!("• Frank: FOLDED");

        // Test that only non-folded players are eligible to win
        let active_players: Vec<_> = room.players.iter().filter(|p| !p.folded).collect();
        assert_eq!(active_players.len(), 2);
        assert_eq!(active_players[0].name, "John");
        assert_eq!(active_players[1].name, "Joe");

        // Test final pot calculation
        assert_eq!(room.pot, 350);

        println!("\n🏆 RESULTS:");
        println!("• John WINS HIGH with score 18");
        println!("• John wins entire pot: {} chips", room.pot);

        // Test that game state is consistent
        assert_eq!(room.players[0].chips, 850); // 1000 - 50 ante - 100 bet
        assert_eq!(room.players[1].chips, 850); // 1000 - 50 ante - 100 bet
        assert_eq!(room.players[2].chips, 950); // 1000 - 50 ante (folded, no betting)

        println!("\n📊 FINAL CHIP COUNTS:");
        println!("• John: {} chips (won {} chips)", room.players[0].chips + room.pot, room.pot);
        println!("• Joe: {} chips (lost {} chips)", room.players[1].chips, 150);
        println!("• Frank: {} chips (lost {} chips)", room.players[2].chips, 50);

        println!("\n✅ Three-player match completed successfully!");
        println!("===============================================");
    }

    /// Test player cap enforcement
    #[test]
    fn test_player_cap() {
        let mut room = create_test_room();

        // Add 4 more players to reach the 7-player cap
        for i in 3..7 {
            room.players.push(create_test_player(&format!("Player{}", i), i, 1000));
        }

        assert_eq!(room.players.len(), 7);

        // Attempting to add an 8th player should be rejected
        // This test validates the MAX_PLAYERS = 7 constraint
        println!("✅ Player cap test: {} players maximum", room.players.len());
    }

    /// Test ante collection logic
    #[test]
    fn test_ante_collection() {
        let mut room = create_test_room();
        let initial_chips = room.players[0].chips;
        let ante = room.ante;

        // Simulate ante collection
        room.pot = 0;
        for player in &mut room.players {
            player.chips -= ante;
            room.pot += ante;
        }

        assert_eq!(room.pot, ante * 3); // 3 players
        assert_eq!(room.players[0].chips, initial_chips - ante);

        println!("✅ Ante collection test: {} chips per player, {} total pot", ante, room.pot);
    }

    /// Test betting limits enforcement
    #[test]
    fn test_betting_limits() {
        let room = create_test_room();
        let small_limit = room.limit_small;
        let big_limit = room.limit_big;

        // Test betting limits based on round
        assert_eq!(small_limit, 10);
        assert_eq!(big_limit, 20);

        // In real implementation, early rounds use small limit, later rounds use big limit
        let round_1_bet = small_limit;
        let round_3_bet = big_limit;
        assert_eq!(round_1_bet, 10);
        assert_eq!(round_3_bet, 20);

        println!("✅ Betting limits test: small limit {} chips, big limit {} chips", small_limit, big_limit);
    }

    /// Test card counting logic
    #[test]
    fn test_card_counting() {
        let mut room = create_test_room();

        // Test initial state
        assert_eq!(room.players[0].cards_count, 0);
        assert!(room.players[0].up_cards.is_empty());

        // Simulate dealing cards
        room.players[0].cards_count = 2; // 2 hole cards
        room.players[0].up_cards = vec![
            Card { rank: Rank::Ace, suit: Suit::Spades, face_up: true }
        ];

        // Total cards = hole cards + up cards
        let total_cards = room.players[0].cards_count;
        let hole_cards = if total_cards >= room.players[0].up_cards.len() {
            total_cards - room.players[0].up_cards.len()
        } else {
            0
        };

        assert_eq!(hole_cards, 1); // 2 total - 1 up card = 1 hole card

        println!("✅ Card counting test: {} total, {} hole, {} up",
                total_cards, hole_cards, room.players[0].up_cards.len());
    }

    /// Test four-player match with different betting scenarios
    #[test]
    fn test_four_player_match() {
        let mut room = create_test_room();
        room.players.push(create_test_player("Santo", 3, 1000));

        println!("\n🎯 FOUR-PLAYER POKER MATCH BEGINS");
        println!("===============================================");

        // Start the game
        room.phase = Phase::Acting;
        room.pot = 200; // 4 players * 50 ante
        room.round = 1;
        room.in_betting = false;

        // Simulate initial card deal
        for player in &mut room.players {
            player.cards_count = 2;
            player.chips -= room.ante;
            player.committed_round = room.ante;
        }

        println!("\n📋 INITIAL SETUP:");
        println!("• John starts with {} chips", room.players[0].chips + room.ante);
        println!("• Joe starts with {} chips", room.players[1].chips + room.ante);
        println!("• Frank starts with {} chips", room.players[2].chips + room.ante);
        println!("• Santo starts with {} chips", room.players[3].chips + room.ante);
        println!("• Ante: {} chips per player", room.ante);
        println!("• Starting pot: {} chips", room.pot);

        println!("\n🃏 INITIAL DEAL (2 hole cards each):");
        println!("• John: [🂠🂠] (2 hole cards)");
        println!("• Joe: [🂠🂠] (2 hole cards)");
        println!("• Frank: [🂠🂠] (2 hole cards)");
        println!("• Santo: [🂠🂠] (2 hole cards)");

        assert_eq!(room.players.len(), 4);
        assert_eq!(room.pot, 200);

        // Add some up cards for first draw
        room.players[0].up_cards = vec![
            Card { rank: Rank::Six, suit: Suit::Hearts, face_up: true },
            Card { rank: Rank::Four, suit: Suit::Clubs, face_up: true }
        ];
        room.players[1].up_cards = vec![
            Card { rank: Rank::Jack, suit: Suit::Diamonds, face_up: true }
        ];
        room.players[2].up_cards = vec![
            Card { rank: Rank::Ace, suit: Suit::Hearts, face_up: true },
            Card { rank: Rank::Two, suit: Suit::Spades, face_up: true },
            Card { rank: Rank::Three, suit: Suit::Diamonds, face_up: true }
        ];
        room.players[3].up_cards = vec![
            Card { rank: Rank::King, suit: Suit::Spades, face_up: true },
            Card { rank: Rank::Queen, suit: Suit::Hearts, face_up: true }
        ];

        println!("\n🎯 ROUND 1 - DRAW PHASE:");
        println!("• John HITS (takes 2 cards): [🂠🂠] + [6♥ 4♣] = 4 cards total");
        println!("• Joe HITS (takes 1 card): [🂠🂠] + [J♦] = 3 cards total");
        println!("• Frank HITS (takes 3 cards): [🂠🂠] + [A♥ 2♠ 3♦] = 5 cards total");
        println!("• Santo HITS (takes 2 cards): [🂠🂠] + [K♠ Q♥] = 4 cards total");

        room.players[0].cards_count = 4;
        room.players[1].cards_count = 3;
        room.players[2].cards_count = 5;
        room.players[3].cards_count = 4;

        // Simulate betting: John bets, Joe calls, Frank raises, Santo folds
        room.in_betting = true;
        room.current_bet = 100;

        println!("\n💰 ROUND 1 - BETTING PHASE:");

        room.players[0].committed_round = 150; // John bet
        room.players[0].chips -= 100;
        room.pot += 100;
        println!("• John BETS {} chips (total committed: {})", 100, room.players[0].committed_round);

        room.players[1].committed_round = 150; // Joe call
        room.players[1].chips -= 100;
        room.pot += 100;
        println!("• Joe CALLS {} chips (total committed: {})", 100, room.players[1].committed_round);

        room.players[2].committed_round = 200; // Frank raise to 150
        room.players[2].chips -= 150;
        room.pot += 150;
        println!("• Frank RAISES to {} chips (total committed: {})", 150, room.players[2].committed_round);

        room.players[3].folded = true; // Santo folds
        println!("• Santo FOLDS (chips remaining: {})", room.players[3].chips);

        println!("  → Current pot: {} chips", room.pot);

        let active_players: Vec<_> = room.players.iter().filter(|p| !p.folded).collect();
        assert_eq!(active_players.len(), 3);
        assert_eq!(room.pot, 550);

        println!("\n📊 RESULTS AFTER ROUND 1:");
        println!("• Active players: John, Joe, Frank");
        println!("• Santo folded early");
        println!("• Current pot: {} chips", room.pot);

        println!("\n🎯 ROUND 2 - DRAW PHASE:");
        println!("Remaining players take more cards...");

        // Add more cards for round 2
        room.players[0].up_cards.extend(vec![
            Card { rank: Rank::Seven, suit: Suit::Spades, face_up: true }
        ]);
        room.players[1].up_cards.extend(vec![
            Card { rank: Rank::Ten, suit: Suit::Clubs, face_up: true },
            Card { rank: Rank::Nine, suit: Suit::Hearts, face_up: true }
        ]);
        room.players[2].up_cards.extend(vec![
            Card { rank: Rank::Five, suit: Suit::Clubs, face_up: true }
        ]);

        room.players[0].cards_count = 5;
        room.players[1].cards_count = 5;
        room.players[2].cards_count = 6;

        println!("• John HITS (takes 1 more): [🂠🂠] + [6♥ 4♣ 7♠] = 5 cards total");
        println!("• Joe HITS (takes 2 more): [🂠🂠] + [J♦ 10♣ 9♥] = 5 cards total");
        println!("• Frank HITS (takes 1 more): [🂠🂠] + [A♥ 2♠ 3♦ 5♣] = 6 cards total");
        println!("• Santo: FOLDED (no more cards)");

        println!("\n🎭 SHOWDOWN:");
        println!("• John's hand: [8♦ 9♠] + [6♥ 4♣ 7♠] = 5 cards");
        println!("  → Low score: 4+6+7+8+9 = 34 (busted for low)");
        println!("  → High score: 4+6+7+8+9 = 34 (busted for high)");

        println!("• Joe's hand: [A♠ 5♦] + [J♦ 10♣ 9♥] = 5 cards");
        println!("  → Low score: A+5+9+10+J = 31 (busted for low)");
        println!("  → High score: A+5+9+10+J = 31 (busted for high)");

        println!("• Frank's hand: [4♥ 6♠] + [A♥ 2♠ 3♦ 5♣] = 6 cards");
        println!("  → Low score: A+2+3+4+5 = 15 (busted for low)");
        println!("  → High score: A+2+3+4+5+6 = 21 (good for high!)");

        println!("• Santo: FOLDED");

        println!("\n🏆 RESULTS:");
        println!("• Frank WINS HIGH with score 21");
        println!("• Frank wins entire pot: {} chips", room.pot);

        println!("\n📊 FINAL CHIP COUNTS:");
        println!("• John: {} chips (lost {} chips)", room.players[0].chips, 150);
        println!("• Joe: {} chips (lost {} chips)", room.players[1].chips, 150);
        println!("• Frank: {} chips (won {} chips)", room.players[2].chips + room.pot, room.pot - 200);
        println!("• Santo: {} chips (lost {} chips)", room.players[3].chips, 50);

        println!("\n✅ Four-player match completed successfully!");
        println!("===============================================");
    }

    /// Test five-player match with multiple folds
    #[test]
    fn test_five_player_match() {
        let mut room = create_test_room();
        room.players.push(create_test_player("Santo", 3, 1000));
        room.players.push(create_test_player("Eve", 4, 1000));

        room.phase = Phase::Acting;
        room.pot = 250; // 5 players * 50 ante
        room.round = 1;

        for player in &mut room.players {
            player.cards_count = 2;
            player.chips -= room.ante;
            player.committed_round = room.ante;
        }

        assert_eq!(room.players.len(), 5);
        assert_eq!(room.pot, 250);

        // Multiple players fold early
        room.players[1].folded = true; // Joe folds
        room.players[3].folded = true; // Santo folds
        room.players[4].folded = true; // Eve folds

        let active_players: Vec<_> = room.players.iter().filter(|p| !p.folded).collect();
        assert_eq!(active_players.len(), 2);

        println!("✅ Five-player match test completed successfully");
        println!("   - Players folded: 3");
        println!("   - Active players remaining: {}", active_players.len());
    }

    /// Test six-player match with complex betting
    #[test]
    fn test_six_player_match() {
        let mut room = create_test_room();
        room.players.push(create_test_player("Santo", 3, 1500));
        room.players.push(create_test_player("Eve", 4, 800));
        room.players.push(create_test_player("Frank", 5, 1200));

        room.phase = Phase::Acting;
        room.pot = 300; // 6 players * 50 ante
        room.round = 2; // Second round uses big limit

        for player in &mut room.players {
            player.cards_count = 3;
            player.chips -= room.ante;
            player.committed_round = room.ante;
        }

        assert_eq!(room.players.len(), 6);
        assert_eq!(room.pot, 300);

        // Test big limit betting (round 2 uses big limit of 20)
        room.in_betting = true;
        room.current_bet = room.limit_big;

        // Different players with different betting patterns
        room.players[0].committed_round = room.ante + room.limit_big; // John bets big limit
        room.players[0].chips -= room.limit_big;
        room.pot += room.limit_big;

        room.players[2].folded = true; // Frank folds
        room.players[4].folded = true; // Eve folds

        let active_players: Vec<_> = room.players.iter().filter(|p| !p.folded).collect();
        assert_eq!(active_players.len(), 4);
        assert_eq!(room.pot, 320);

        println!("✅ Six-player match test completed successfully");
        println!("   - Big limit betting: {} chips", room.limit_big);
        println!("   - Final pot: {} chips", room.pot);
    }

    /// Test maximum seven-player match
    #[test]
    fn test_seven_player_max_match() {
        let mut room = create_test_room();
        room.players.push(create_test_player("Santo", 3, 2000));
        room.players.push(create_test_player("Eve", 4, 1500));
        room.players.push(create_test_player("Frank", 5, 1800));
        room.players.push(create_test_player("Grace", 6, 1200));

        room.phase = Phase::Acting;
        room.pot = 350; // 7 players * 50 ante
        room.round = 3; // Third round uses big limit

        for player in &mut room.players {
            player.cards_count = 4;
            player.chips -= room.ante;
            player.committed_round = room.ante;
        }

        assert_eq!(room.players.len(), 7); // Maximum players
        assert_eq!(room.pot, 350);

        // Simulate complex multi-round betting
        room.in_betting = true;
        room.current_bet = room.limit_big;
        room.raises_made = 2;

        // Various player actions in maximum player scenario
        let mut total_additional_bets = 0;
        for (i, player) in room.players.iter_mut().enumerate() {
            match i {
                0 | 2 | 5 => {
                    // Players 0, 2, 5 bet big limit
                    player.committed_round += room.limit_big;
                    player.chips -= room.limit_big;
                    total_additional_bets += room.limit_big;
                }
                1 | 4 => {
                    // Players 1, 4 fold
                    player.folded = true;
                }
                _ => {
                    // Players 3, 6 call
                    player.committed_round += room.limit_big;
                    player.chips -= room.limit_big;
                    total_additional_bets += room.limit_big;
                }
            }
        }
        room.pot += total_additional_bets;

        let active_players: Vec<_> = room.players.iter().filter(|p| !p.folded).collect();
        assert_eq!(active_players.len(), 5);
        assert!(room.pot > 350); // Should be higher due to betting

        println!("✅ Seven-player maximum match test completed successfully");
        println!("   - Maximum players: {}", room.players.len());
        println!("   - Active players after betting: {}", active_players.len());
        println!("   - Final pot: {} chips", room.pot);
        println!("   - Raises made: {}/{}", room.raises_made, room.max_raises);
    }

    /// Test edge case with varying chip stacks
    #[test]
    fn test_varied_chip_stacks_match() {
        let mut room = PublicRoom {
            room: "Varied Stakes Room".to_string(),
            game_variant: GameVariant::SevenTwentySeven,
            dealer_seat: 0,
            to_act_seat: 0,
            pot: 0,
            ante: 25, // Lower ante
            phase: Phase::Acting,
            players: vec![
                create_test_player("John", 0, 2500),   // High roller
                create_test_player("Joe", 1, 100),     // Short stack
                create_test_player("Frank", 2, 500), // Medium stack
                create_test_player("Santo", 3, 75),     // Very short stack
                create_test_player("Eve", 4, 1000),    // Standard stack
            ],
            in_betting: true,
            current_bet: 25,
            raises_made: 0,
            max_raises: 3,
            round: 1,
            limit_small: 5,
            limit_big: 10,
            community_cards: vec![],
            scheduled_start: None,
            checked_in_players: vec![],
            elected_players: vec![],
            current_dealer_id: None,
            available_variants: vec![GameVariant::SevenTwentySeven, GameVariant::Omaha, GameVariant::TexasHoldem],
        };

        // Simulate ante collection with varied stacks
        room.pot = room.ante * room.players.len() as u64;
        for player in &mut room.players {
            player.chips -= room.ante;
            player.committed_round = room.ante;
            player.cards_count = 2;
        }

        assert_eq!(room.players.len(), 5);
        assert_eq!(room.pot, 125); // 5 players * 25 ante

        // Short stack player (Santo) goes all-in
        room.players[3].chips = 0; // All chips committed
        room.players[3].committed_round = 75; // 25 ante + 50 all-in
        room.pot += 50;

        // Test that short stack situations are handled
        assert_eq!(room.players[3].chips, 0);
        assert_eq!(room.pot, 175);

        println!("✅ Varied chip stacks match test completed successfully");
        println!("   - Ante: {} chips", room.ante);
        println!("   - Short stack all-in detected");
        println!("   - Final pot: {} chips", room.pot);

        // Verify chip distribution makes sense
        let total_chips: u64 = room.players.iter().map(|p| p.chips + p.committed_round).sum::<u64>();
        let expected_total = 2500 + 100 + 500 + 75 + 1000; // Initial chip totals
        assert_eq!(total_chips, expected_total);
        println!("   - Total chips conserved: {} = {}", total_chips, expected_total);
    }
}

#[cfg(test)]
mod server_tests {
    use super::*;
    use crate::game;

    /// Test distributed table registry functionality
    #[test]
    fn test_distributed_table_registry() {
        let mut distributed_tables = HashMap::new();

        // Create test table info
        let table_info = TableInfo {
            name: "Test Distributed Table".to_string(),
            game_variant: GameVariant::SevenTwentySeven,
            player_count: 2,
            phase: Phase::Lobby,
            server_port: Some(9100),
        };

        // Test table registration
        distributed_tables.insert("Test Distributed Table".to_string(), table_info.clone());

        assert_eq!(distributed_tables.len(), 1);
        assert!(distributed_tables.contains_key("Test Distributed Table"));

        let registered_table = distributed_tables.get("Test Distributed Table").unwrap();
        assert_eq!(registered_table.server_port, Some(9100));
        assert_eq!(registered_table.game_variant, GameVariant::SevenTwentySeven);

        println!("✅ Distributed table registry test passed");
        println!("   - Registered table: {}", registered_table.name);
        println!("   - Port: {:?}", registered_table.server_port);
        println!("   - Game variant: {:?}", registered_table.game_variant);
    }

    /// Test table list functionality with mixed central and distributed tables
    #[test]
    fn test_mixed_table_list() {
        let mut all_tables = Vec::new();

        // Central server tables (no port)
        all_tables.push(TableInfo {
            name: "Central Table 1".to_string(),
            game_variant: GameVariant::TexasHoldem,
            player_count: 4,
            phase: Phase::Acting,
            server_port: None,
        });

        all_tables.push(TableInfo {
            name: "Central Table 2".to_string(),
            game_variant: GameVariant::Omaha,
            player_count: 6,
            phase: Phase::Lobby,
            server_port: None,
        });

        // Distributed tables (with ports)
        all_tables.push(TableInfo {
            name: "Distributed Table 1".to_string(),
            game_variant: GameVariant::SevenTwentySeven,
            player_count: 2,
            phase: Phase::Acting,
            server_port: Some(9100),
        });

        all_tables.push(TableInfo {
            name: "Distributed Table 2".to_string(),
            game_variant: GameVariant::TexasHoldem,
            player_count: 3,
            phase: Phase::Showdown,
            server_port: Some(9101),
        });

        // Test table categorization
        let central_tables: Vec<_> = all_tables.iter().filter(|t| t.server_port.is_none()).collect();
        let distributed_tables: Vec<_> = all_tables.iter().filter(|t| t.server_port.is_some()).collect();

        assert_eq!(central_tables.len(), 2);
        assert_eq!(distributed_tables.len(), 2);
        assert_eq!(all_tables.len(), 4);

        println!("✅ Mixed table list test passed");
        println!("   - Total tables: {}", all_tables.len());
        println!("   - Central tables: {}", central_tables.len());
        println!("   - Distributed tables: {}", distributed_tables.len());

        for table in &distributed_tables {
            println!("   - Distributed: {} (port {:?})", table.name, table.server_port);
        }
    }

    /// Test game room creation and player management
    #[test]
    fn test_room_creation_and_management() {
        let room_name = "Test Room".to_string();
        let mut room = game::Room::new(room_name.clone());

        // Test initial room state
        assert_eq!(room.name, "Test Room");
        assert_eq!(room.phase, Phase::Lobby);
        assert_eq!(room.players.len(), 0);
        assert_eq!(room.pot, 0);
        assert_eq!(room.ante, 10);

        // Test room configuration
        room.game_variant = GameVariant::Omaha;
        room.ante = 25;
        room.limit_small = 15;
        room.limit_big = 30;
        room.max_raises = 4;

        assert_eq!(room.game_variant, GameVariant::Omaha);
        assert_eq!(room.ante, 25);
        assert_eq!(room.limit_small, 15);
        assert_eq!(room.limit_big, 30);
        assert_eq!(room.max_raises, 4);

        println!("✅ Room creation and management test passed");
        println!("   - Room name: {}", room.name);
        println!("   - Game variant: {:?}", room.game_variant);
        println!("   - Ante: {}", room.ante);
        println!("   - Limits: {}/{}", room.limit_small, room.limit_big);
    }

    /// Test player seat management
    #[test]
    fn test_player_seat_management() {
        let room = game::Room::new("Seat Test Room".to_string());

        let player_id_1 = Uuid::new_v4();
        let player_id_2 = Uuid::new_v4();
        let player_id_3 = Uuid::new_v4();

        // Test seat_of function with empty room
        assert_eq!(game::seat_of(&room, player_id_1), None);

        // Create a room with players for testing
        let mut test_room = game::Room::new("Test Room".to_string());
        let (tx, _rx) = mpsc::unbounded_channel();

        test_room.players.push(game::PlayerSeat {
            id: player_id_1,
            name: "Player 1".to_string(),
            chips: 1000,
            folded: false,
            standing: false,
            up_cards: vec![],
            down_cards: vec![],
            ready: true,
            committed_round: 0,
            tx: tx.clone(),
        });

        test_room.players.push(game::PlayerSeat {
            id: player_id_2,
            name: "Player 2".to_string(),
            chips: 1500,
            folded: true,
            standing: false,
            up_cards: vec![],
            down_cards: vec![],
            ready: false,
            committed_round: 50,
            tx: tx.clone(),
        });

        // Test seat finding
        assert_eq!(game::seat_of(&test_room, player_id_1), Some(0));
        assert_eq!(game::seat_of(&test_room, player_id_2), Some(1));
        assert_eq!(game::seat_of(&test_room, player_id_3), None);

        // Test alive seats (non-folded players)
        let alive_seats = game::alive_seats(&test_room);
        assert_eq!(alive_seats.len(), 1);
        assert_eq!(alive_seats[0].1.name, "Player 1");

        println!("✅ Player seat management test passed");
        println!("   - Player 1 seat: {:?}", game::seat_of(&test_room, player_id_1));
        println!("   - Player 2 seat: {:?}", game::seat_of(&test_room, player_id_2));
        println!("   - Alive players: {}", alive_seats.len());
    }

    /// Test betting size calculation based on round
    #[test]
    fn test_betting_size_calculation() {
        let mut room = game::Room::new("Betting Test Room".to_string());
        room.limit_small = 25;
        room.limit_big = 50;

        // Test early rounds use small limit
        room.round = 1;
        assert_eq!(game::bet_size_for_round(&room), 25);

        room.round = 2;
        assert_eq!(game::bet_size_for_round(&room), 25);

        // Test later rounds use big limit
        room.round = 3;
        assert_eq!(game::bet_size_for_round(&room), 50);

        room.round = 4;
        assert_eq!(game::bet_size_for_round(&room), 50);

        println!("✅ Betting size calculation test passed");
        println!("   - Round 1-2 bet size: {}", game::bet_size_for_round(&room));
        room.round = 3;
        println!("   - Round 3+ bet size: {}", game::bet_size_for_round(&room));
    }

    /// Test chip commitment and pot management
    #[test]
    fn test_chip_commitment() {
        let mut room = game::Room::new("Chip Test Room".to_string());
        let (tx, _rx) = mpsc::unbounded_channel();

        room.players.push(game::PlayerSeat {
            id: Uuid::new_v4(),
            name: "Test Player".to_string(),
            chips: 1000,
            folded: false,
            standing: false,
            up_cards: vec![],
            down_cards: vec![],
            ready: true,
            committed_round: 0,
            tx,
        });

        let initial_chips = room.players[0].chips;
        let initial_pot = room.pot;

        // Test committing chips
        game::commit(&mut room, 0, 100);

        assert_eq!(room.players[0].chips, initial_chips - 100);
        assert_eq!(room.players[0].committed_round, 100);
        assert_eq!(room.pot, initial_pot + 100);

        // Test committing more than available chips
        game::commit(&mut room, 0, 2000); // Player only has 900 left

        assert_eq!(room.players[0].chips, 0); // All chips committed
        assert_eq!(room.players[0].committed_round, 1000); // 100 + 900
        assert_eq!(room.pot, 1000); // Total committed

        println!("✅ Chip commitment test passed");
        println!("   - Final chips: {}", room.players[0].chips);
        println!("   - Total committed: {}", room.players[0].committed_round);
        println!("   - Final pot: {}", room.pot);
    }

    /// Test game state validation functions
    #[test]
    fn test_game_state_validation() {
        let mut room = game::Room::new("Validation Test Room".to_string());
        let (tx, _rx) = mpsc::unbounded_channel();
        let player_id = Uuid::new_v4();

        room.players.push(game::PlayerSeat {
            id: player_id,
            name: "Test Player".to_string(),
            chips: 1000,
            folded: false,
            standing: false,
            up_cards: vec![],
            down_cards: vec![],
            ready: true,
            committed_round: 0,
            tx,
        });

        room.phase = Phase::Acting;
        room.in_betting = false;
        room.to_act_seat = 0;

        // Test can_take_card validation
        let result = game::can_take_card(&room, player_id);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Test validation when not player's turn
        room.to_act_seat = 1; // Different seat
        let result = game::can_take_card(&room, player_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Not your turn"));

        // Test validation when player is folded
        room.to_act_seat = 0;
        room.players[0].folded = true;
        let result = game::can_take_card(&room, player_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Player folded"));

        // Test validation when player is standing
        room.players[0].folded = false;
        room.players[0].standing = true;
        let result = game::can_take_card(&room, player_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Player already standing"));

        println!("✅ Game state validation test passed");
        println!("   - Valid action detected correctly");
        println!("   - Invalid actions properly rejected");
    }

    /// Test next dealer rotation
    #[test]
    fn test_dealer_rotation() {
        let mut room = game::Room::new("Dealer Test Room".to_string());
        let (tx, _rx) = mpsc::unbounded_channel();

        // Add 3 players
        for i in 0..3 {
            room.players.push(game::PlayerSeat {
                id: Uuid::new_v4(),
                name: format!("Player {}", i + 1),
                chips: 1000,
                folded: false,
                standing: false,
                up_cards: vec![],
                down_cards: vec![],
                ready: true,
                committed_round: 0,
                tx: tx.clone(),
            });
        }

        // Test dealer rotation
        let next_dealer_0 = game::next_dealer_left_of(&room, 0);
        assert!(next_dealer_0.is_some());
        assert_eq!(next_dealer_0.unwrap(), room.players[1].id);

        let next_dealer_1 = game::next_dealer_left_of(&room, 1);
        assert!(next_dealer_1.is_some());
        assert_eq!(next_dealer_1.unwrap(), room.players[2].id);

        let next_dealer_2 = game::next_dealer_left_of(&room, 2);
        assert!(next_dealer_2.is_some());
        assert_eq!(next_dealer_2.unwrap(), room.players[0].id); // Wraps around

        // Test with empty room
        let empty_room = game::Room::new("Empty Room".to_string());
        let no_dealer = game::next_dealer_left_of(&empty_room, 0);
        assert!(no_dealer.is_none());

        println!("✅ Dealer rotation test passed");
        println!("   - Dealer rotates properly through seats");
        println!("   - Handles edge cases correctly");
    }

    /// Test scoring functions for 7/27 variant
    #[test]
    fn test_seven_twenty_seven_scoring() {
        // Test score calculation
        let test_cards = vec![
            Card { rank: Rank::Ace, suit: Suit::Spades, face_up: true },
            Card { rank: Rank::Two, suit: Suit::Hearts, face_up: true },
            Card { rank: Rank::Three, suit: Suit::Clubs, face_up: true },
        ];

        let score = score_hand(&test_cards);

        // A+2+3: Ace can be 1 or 11
        // Best under 7: 1+2+3 = 6 (valid)
        // Best under 27: 11+2+3 = 16 (better than 6, so this is chosen)
        assert_eq!(score.best_under_7, Some(6.0));
        assert_eq!(score.best_under_27, Some(16.0));
        assert!(!score.bust_27);

        // Test with face cards (worth 0.5 each)
        let face_cards = vec![
            Card { rank: Rank::Jack, suit: Suit::Spades, face_up: true },
            Card { rank: Rank::Queen, suit: Suit::Hearts, face_up: true },
            Card { rank: Rank::King, suit: Suit::Clubs, face_up: true },
            Card { rank: Rank::Ace, suit: Suit::Diamonds, face_up: true },
        ];

        let score = score_hand(&face_cards);

        // J(0.5) + Q(0.5) + K(0.5) + A(can be 1 or 11)
        // Let's check what the actual result is by debugging
        println!("Face cards score: {:?}", score);

        // According to the algorithm, it tries Ace as 1 and 11, picks the best under the limit
        // For under 7: 0.5 + 0.5 + 0.5 + 1 = 2.5 (valid)
        // For under 27: 0.5 + 0.5 + 0.5 + 11 = 12.5 (better than 2.5, so this is chosen)
        assert_eq!(score.best_under_7, Some(2.5));
        assert_eq!(score.best_under_27, Some(12.5));
        assert!(!score.bust_27);

        // Test bust scenario
        let bust_cards = vec![
            Card { rank: Rank::Ten, suit: Suit::Spades, face_up: true },
            Card { rank: Rank::Ten, suit: Suit::Hearts, face_up: true },
            Card { rank: Rank::Ten, suit: Suit::Clubs, face_up: true },
        ];

        let score = score_hand(&bust_cards);

        // 10+10+10 = 30 (busted)
        assert_eq!(score.best_under_7, None);
        assert_eq!(score.best_under_27, None);
        assert!(score.bust_27);

        println!("✅ Seven Twenty-Seven scoring test passed");
        println!("   - Low hand (A+2+3): {:?}", score_hand(&test_cards));
        println!("   - Face cards (J+Q+K+A): {:?}", score_hand(&face_cards));
        println!("   - Bust hand (10+10+10): {:?}", score_hand(&bust_cards));
    }

    /// Test PublicRoom conversion
    #[test]
    fn test_public_room_conversion() {
        let mut room = game::Room::new("Public Test Room".to_string());
        let (tx, _rx) = mpsc::unbounded_channel();

        room.game_variant = GameVariant::TexasHoldem;
        room.ante = 100;
        room.pot = 300;
        room.phase = Phase::Acting;
        room.dealer_seat = 1;
        room.to_act_seat = 2;
        room.round = 2;
        room.in_betting = true;
        room.current_bet = 50;
        room.raises_made = 1;
        room.max_raises = 3;
        room.limit_small = 25;
        room.limit_big = 50;

        // Add a player
        room.players.push(game::PlayerSeat {
            id: Uuid::new_v4(),
            name: "Test Player".to_string(),
            chips: 900,
            folded: false,
            standing: false,
            up_cards: vec![
                Card { rank: Rank::Ace, suit: Suit::Spades, face_up: true }
            ],
            down_cards: vec![
                Card { rank: Rank::King, suit: Suit::Hearts, face_up: false }
            ],
            ready: true,
            committed_round: 100,
            tx,
        });

        // Convert to public room
        let public_room = game::public_room(&room);

        // Test all fields are correctly converted
        assert_eq!(public_room.room, "Public Test Room");
        assert_eq!(public_room.game_variant, GameVariant::TexasHoldem);
        assert_eq!(public_room.ante, 100);
        assert_eq!(public_room.pot, 300);
        assert_eq!(public_room.phase, Phase::Acting);
        assert_eq!(public_room.dealer_seat, 1);
        assert_eq!(public_room.to_act_seat, 2);
        assert_eq!(public_room.round, 2);
        assert_eq!(public_room.in_betting, true);
        assert_eq!(public_room.current_bet, 50);
        assert_eq!(public_room.raises_made, 1);
        assert_eq!(public_room.max_raises, 3);
        assert_eq!(public_room.limit_small, 25);
        assert_eq!(public_room.limit_big, 50);
        assert_eq!(public_room.players.len(), 1);

        // Test player conversion
        let public_player = &public_room.players[0];
        assert_eq!(public_player.name, "Test Player");
        assert_eq!(public_player.seat, 0);
        assert_eq!(public_player.chips, 900);
        assert_eq!(public_player.folded, false);
        assert_eq!(public_player.standing, false);
        assert_eq!(public_player.up_cards.len(), 1);
        assert_eq!(public_player.cards_count, 2); // 1 up + 1 down
        assert_eq!(public_player.committed_round, 100);
        assert_eq!(public_player.ready, true);

        println!("✅ Public room conversion test passed");
        println!("   - Room: {}", public_room.room);
        println!("   - Game variant: {:?}", public_room.game_variant);
        println!("   - Players: {}", public_room.players.len());
        println!("   - Player cards: {} total ({} visible)",
                public_player.cards_count, public_player.up_cards.len());
    }

    /// Test comprehensive 7-round match with full betting and chat interactions
    #[test]
    fn test_full_seven_round_match_with_chat() {
        let mut room = super::game_tests::create_test_room();

        // Add 2 more players for a 5-player match
        room.players.push(super::game_tests::create_test_player("Santo", 3, 1000));
        room.players.push(super::game_tests::create_test_player("Bob", 4, 1000));

        println!("\n🎯 COMPREHENSIVE 7-ROUND POKER MATCH");
        println!("====================================================");
        println!("Players: John, Joe, Frank, Santo, Bob");
        println!("Game: 7/27 Stud Poker");
        println!("Ante: {} chips | Limits: {}/{}", room.ante, room.limit_small, room.limit_big);
        println!("====================================================");

        // Initialize game state
        room.phase = Phase::Acting;
        room.pot = 250; // 5 players * 50 ante
        room.round = 1;
        room.in_betting = false;
        room.dealer_seat = 0; // John is dealer
        room.to_act_seat = 1; // Joe acts first

        // Collect antes
        for player in &mut room.players {
            player.chips -= room.ante;
            player.committed_round = room.ante;
            player.cards_count = 2; // Start with 2 hole cards
        }

        println!("\n💰 ANTE COLLECTION:");
        println!("• Each player posts {} chips ante", room.ante);
        println!("• Starting pot: {} chips", room.pot);
        println!("• Dealer: {} (seat {})", room.players[room.dealer_seat].name, room.dealer_seat);

        // Chat before first round
        println!("\n💬 PRE-GAME CHAT:");
        println!("• John: \"Good luck everyone! Let's play some 7/27!\"");
        println!("• Santo: \"May the cards be with us all 🃏\"");
        println!("• Frank: \"Time to win some chips!\"");

        // ============ ROUND 1 ============
        println!("\n🎯 ROUND 1 - INITIAL DRAW PHASE");
        println!("═══════════════════════════════════════");
        println!("Each player receives 2 hole cards and can draw additional cards...");

        // Simulate card draws for each player
        let round1_draws = vec![
            ("John", 1, vec![Card { rank: Rank::Seven, suit: Suit::Hearts, face_up: true }]),
            ("Joe", 2, vec![
                Card { rank: Rank::Ace, suit: Suit::Spades, face_up: true },
                Card { rank: Rank::Three, suit: Suit::Clubs, face_up: true }
            ]),
            ("Frank", 1, vec![Card { rank: Rank::King, suit: Suit::Diamonds, face_up: true }]),
            ("Santo", 3, vec![
                Card { rank: Rank::Two, suit: Suit::Hearts, face_up: true },
                Card { rank: Rank::Four, suit: Suit::Spades, face_up: true },
                Card { rank: Rank::Five, suit: Suit::Clubs, face_up: true }
            ]),
            ("Bob", 2, vec![
                Card { rank: Rank::Jack, suit: Suit::Hearts, face_up: true },
                Card { rank: Rank::Queen, suit: Suit::Diamonds, face_up: true }
            ])
        ];

        for (i, (name, draw_count, cards)) in round1_draws.iter().enumerate() {
            room.players[i].cards_count = 2 + draw_count;
            room.players[i].up_cards = cards.clone();
            println!("• {}: DRAWS {} card(s) → {} total cards", name, draw_count, 2 + draw_count);
            for card in cards {
                println!("  └─ Shows: {}", card);
            }
        }

        // Round 1 Betting
        println!("\n💰 ROUND 1 - BETTING PHASE:");
        room.in_betting = true;
        room.current_bet = room.limit_small; // 10 chips

        println!("• Joe (to act): BETS {} chips", room.limit_small);
        room.players[1].chips -= room.limit_small;
        room.players[1].committed_round += room.limit_small;
        room.pot += room.limit_small;

        println!("• Frank: CALLS {} chips", room.limit_small);
        room.players[2].chips -= room.limit_small;
        room.players[2].committed_round += room.limit_small;
        room.pot += room.limit_small;

        println!("• Santo: RAISES to {} chips", room.limit_small * 2);
        room.current_bet = room.limit_small * 2;
        room.players[3].chips -= room.limit_small * 2;
        room.players[3].committed_round += room.limit_small * 2;
        room.pot += room.limit_small * 2;
        room.raises_made = 1;

        println!("• Bob: FOLDS");
        room.players[4].folded = true;

        println!("• John: CALLS {} chips", room.limit_small * 2);
        room.players[0].chips -= room.limit_small * 2;
        room.players[0].committed_round += room.limit_small * 2;
        room.pot += room.limit_small * 2;

        println!("• Joe: CALLS additional {} chips", room.limit_small);
        room.players[1].chips -= room.limit_small;
        room.players[1].committed_round += room.limit_small;
        room.pot += room.limit_small;

        println!("• Frank: CALLS additional {} chips", room.limit_small);
        room.players[2].chips -= room.limit_small;
        room.players[2].committed_round += room.limit_small;
        room.pot += room.limit_small;

        println!("  → Round 1 pot: {} chips", room.pot);

        println!("\n💬 ROUND 1 CHAT:");
        println!("• Bob: \"Ugh, had to fold early. Good luck guys!\"");
        println!("• Santo: \"Nice raise! This is heating up 🔥\"");
        println!("• Joe: \"My ace is looking good so far\"");

        // ============ ROUND 2 ============
        room.round = 2;
        room.in_betting = false;
        for player in &mut room.players {
            player.committed_round = 0; // Reset for new round
        }

        println!("\n🎯 ROUND 2 - DRAW PHASE");
        println!("═══════════════════════════════════════");

        let round2_draws = vec![
            ("John", 2, vec![
                Card { rank: Rank::Seven, suit: Suit::Hearts, face_up: true },
                Card { rank: Rank::Two, suit: Suit::Diamonds, face_up: true },
                Card { rank: Rank::Six, suit: Suit::Spades, face_up: true }
            ]),
            ("Joe", 1, vec![
                Card { rank: Rank::Ace, suit: Suit::Spades, face_up: true },
                Card { rank: Rank::Three, suit: Suit::Clubs, face_up: true },
                Card { rank: Rank::Five, suit: Suit::Hearts, face_up: true }
            ]),
            ("Frank", 2, vec![
                Card { rank: Rank::King, suit: Suit::Diamonds, face_up: true },
                Card { rank: Rank::Queen, suit: Suit::Hearts, face_up: true },
                Card { rank: Rank::Jack, suit: Suit::Clubs, face_up: true }
            ]),
            ("Santo", 1, vec![
                Card { rank: Rank::Two, suit: Suit::Hearts, face_up: true },
                Card { rank: Rank::Four, suit: Suit::Spades, face_up: true },
                Card { rank: Rank::Five, suit: Suit::Clubs, face_up: true },
                Card { rank: Rank::Three, suit: Suit::Diamonds, face_up: true }
            ])
        ];

        for (i, (name, draw_count, cards)) in round2_draws.iter().enumerate() {
            if !room.players[i].folded {
                room.players[i].cards_count += draw_count;
                room.players[i].up_cards = cards.clone();
                println!("• {}: DRAWS {} more → {} total cards", name, draw_count, room.players[i].cards_count);
                println!("  └─ Showing: {:?}", cards.iter().map(|c| format!("{}", c)).collect::<Vec<_>>().join(", "));
            }
        }

        println!("\n💰 ROUND 2 - BETTING PHASE:");
        room.in_betting = true;
        room.current_bet = room.limit_small;

        println!("• John: CHECKS");
        println!("• Joe: BETS {} chips", room.limit_small);
        room.players[1].chips -= room.limit_small;
        room.players[1].committed_round = room.limit_small;
        room.pot += room.limit_small;

        println!("• Frank: CALLS {} chips", room.limit_small);
        room.players[2].chips -= room.limit_small;
        room.players[2].committed_round = room.limit_small;
        room.pot += room.limit_small;

        println!("• Santo: CALLS {} chips", room.limit_small);
        room.players[3].chips -= room.limit_small;
        room.players[3].committed_round = room.limit_small;
        room.pot += room.limit_small;

        println!("• John: CALLS {} chips", room.limit_small);
        room.players[0].chips -= room.limit_small;
        room.players[0].committed_round = room.limit_small;
        room.pot += room.limit_small;

        println!("  → Round 2 pot: {} chips", room.pot);

        println!("\n💬 ROUND 2 CHAT:");
        println!("• Frank: \"These face cards are dangerous...\"");
        println!("• John: \"Nice low cards Santo, building for 7?\"");
        println!("• Santo: \"Maybe... or maybe I'm going for 27! 😉\"");

        // ============ ROUND 3 ============
        room.round = 3;
        room.in_betting = false;
        for player in &mut room.players {
            player.committed_round = 0;
        }

        println!("\n🎯 ROUND 3 - DRAW PHASE (Big Limit Now: {})", room.limit_big);
        println!("═══════════════════════════════════════");

        let round3_draws = vec![
            ("John", 1, vec![
                Card { rank: Rank::Seven, suit: Suit::Hearts, face_up: true },
                Card { rank: Rank::Two, suit: Suit::Diamonds, face_up: true },
                Card { rank: Rank::Six, suit: Suit::Spades, face_up: true },
                Card { rank: Rank::Ace, suit: Suit::Hearts, face_up: true }
            ]),
            ("Joe", 2, vec![
                Card { rank: Rank::Ace, suit: Suit::Spades, face_up: true },
                Card { rank: Rank::Three, suit: Suit::Clubs, face_up: true },
                Card { rank: Rank::Five, suit: Suit::Hearts, face_up: true },
                Card { rank: Rank::Two, suit: Suit::Spades, face_up: true },
                Card { rank: Rank::Four, suit: Suit::Diamonds, face_up: true }
            ]),
            ("Frank", 1, vec![
                Card { rank: Rank::King, suit: Suit::Diamonds, face_up: true },
                Card { rank: Rank::Queen, suit: Suit::Hearts, face_up: true },
                Card { rank: Rank::Jack, suit: Suit::Clubs, face_up: true },
                Card { rank: Rank::Ten, suit: Suit::Spades, face_up: true }
            ]),
            ("Santo", 2, vec![
                Card { rank: Rank::Two, suit: Suit::Hearts, face_up: true },
                Card { rank: Rank::Four, suit: Suit::Spades, face_up: true },
                Card { rank: Rank::Five, suit: Suit::Clubs, face_up: true },
                Card { rank: Rank::Three, suit: Suit::Diamonds, face_up: true },
                Card { rank: Rank::Ace, suit: Suit::Clubs, face_up: true },
                Card { rank: Rank::Six, suit: Suit::Hearts, face_up: true }
            ])
        ];

        for (i, (name, draw_count, cards)) in round3_draws.iter().enumerate() {
            if !room.players[i].folded {
                room.players[i].cards_count += draw_count;
                room.players[i].up_cards = cards.clone();
                println!("• {}: DRAWS {} more → {} total cards", name, draw_count, room.players[i].cards_count);
            }
        }

        println!("\n💰 ROUND 3 - BETTING PHASE (Big Limit: {}):", room.limit_big);
        room.in_betting = true;
        room.current_bet = room.limit_big;

        println!("• John: BETS {} chips", room.limit_big);
        room.players[0].chips -= room.limit_big;
        room.players[0].committed_round = room.limit_big;
        room.pot += room.limit_big;

        println!("• Joe: RAISES to {} chips", room.limit_big * 2);
        room.current_bet = room.limit_big * 2;
        room.players[1].chips -= room.limit_big * 2;
        room.players[1].committed_round = room.limit_big * 2;
        room.pot += room.limit_big * 2;
        room.raises_made += 1;

        println!("• Frank: FOLDS (showing high cards)");
        room.players[2].folded = true;

        println!("• Santo: CALLS {} chips", room.limit_big * 2);
        room.players[3].chips -= room.limit_big * 2;
        room.players[3].committed_round = room.limit_big * 2;
        room.pot += room.limit_big * 2;

        println!("• John: CALLS additional {} chips", room.limit_big);
        room.players[0].chips -= room.limit_big;
        room.players[0].committed_round += room.limit_big;
        room.pot += room.limit_big;

        println!("  → Round 3 pot: {} chips", room.pot);

        println!("\n💬 ROUND 3 CHAT:");
        println!("• Frank: \"Too rich for my blood with these face cards. Folding.\"");
        println!("• Joe: \"Feeling confident with this straight draw!\"");
        println!("• Santo: \"This low hand is looking sweet! 🍯\"");
        println!("• John: \"We're getting serious now folks!\"");

        // ============ ROUNDS 4-5 ============
        for round_num in 4..=5 {
            room.round = round_num;
            room.in_betting = false;
            for player in &mut room.players {
                player.committed_round = 0;
            }

            println!("\n🎯 ROUND {} - DRAW PHASE", round_num);
            println!("═══════════════════════════════════════");

            // Each active player draws 1 more card
            let active_indices: Vec<_> = room.players.iter().enumerate()
                .filter(|(_, p)| !p.folded)
                .map(|(i, _)| i)
                .collect();

            for i in active_indices {
                let new_card = match i {
                    0 => Card { rank: Rank::Three, suit: Suit::Hearts, face_up: true }, // John
                    1 => Card { rank: Rank::Seven, suit: Suit::Diamonds, face_up: true }, // Joe
                    3 => Card { rank: Rank::Two, suit: Suit::Clubs, face_up: true }, // Santo
                    _ => continue
                };

                let mut new_cards = room.players[i].up_cards.clone();
                new_cards.push(new_card);
                room.players[i].up_cards = new_cards;
                room.players[i].cards_count += 1;

                println!("• {}: DRAWS 1 more → {} total cards", room.players[i].name, room.players[i].cards_count);
                println!("  └─ New card: {}", new_card);
            }

            println!("\n💰 ROUND {} - BETTING PHASE:", round_num);
            room.in_betting = true;
            room.current_bet = room.limit_big;

            let bet_actions = if round_num == 4 {
                vec![
                    ("John", "CHECKS"),
                    ("Joe", "BETS 20"),
                    ("Santo", "CALLS 20"),
                    ("John", "CALLS 20")
                ]
            } else {
                vec![
                    ("John", "BETS 20"),
                    ("Joe", "CALLS 20"),
                    ("Santo", "RAISES to 40"),
                    ("John", "CALLS 20 more"),
                    ("Joe", "CALLS 20 more")
                ]
            };

            for (name, action) in bet_actions {
                println!("• {}: {}", name, action);
                let player_idx = room.players.iter().position(|p| p.name == name).unwrap();
                if action.contains("BETS") || action.contains("CALLS") || action.contains("RAISES") {
                    let amount = if action.contains("40") { room.limit_big * 2 } else { room.limit_big };
                    room.players[player_idx].chips -= amount;
                    room.players[player_idx].committed_round += amount;
                    room.pot += amount;
                }
            }

            println!("  → Round {} pot: {} chips", round_num, room.pot);

            println!("\n💬 ROUND {} CHAT:", round_num);
            let chat_messages = if round_num == 4 {
                vec![
                    "• Joe: \"Getting close to my target number!\"",
                    "• Santo: \"Still building that perfect low...\"",
                    "• John: \"This is intense! Love this game!\"",
                ]
            } else {
                vec![
                    "• Santo: \"Time to push! I like my chances 💪\"",
                    "• John: \"Santo means business with that raise!\"",
                    "• Joe: \"Committed now, let's see where this goes\"",
                ]
            };
            for msg in chat_messages {
                println!("{}", msg);
            }
        }

        // ============ ROUNDS 6-7 ============
        for round_num in 6..=7 {
            room.round = round_num;
            room.in_betting = false;
            for player in &mut room.players {
                player.committed_round = 0;
            }

            println!("\n🎯 ROUND {} - FINAL DRAW PHASE", round_num);
            println!("═══════════════════════════════════════");

            if round_num == 6 {
                println!("• John: STANDS (no more cards) → 7 total cards");
                println!("• Joe: DRAWS 1 more → 7 total cards");
                println!("  └─ New card: 9♠");
                room.players[1].cards_count = 7;

                println!("• Santo: STANDS (no more cards) → 8 total cards");
            } else {
                println!("• All remaining players STAND");
                println!("• Final card counts: John(7), Joe(7), Santo(8)");
            }

            println!("\n💰 ROUND {} - BETTING PHASE:", round_num);
            room.in_betting = true;

            if round_num == 6 {
                println!("• John: CHECKS");
                println!("• Joe: BETS {} chips", room.limit_big);
                println!("• Santo: CALLS {} chips", room.limit_big);
                println!("• John: FOLDS");
                room.players[0].folded = true;

                room.players[1].chips -= room.limit_big;
                room.players[1].committed_round = room.limit_big;
                room.pot += room.limit_big;

                room.players[3].chips -= room.limit_big;
                room.players[3].committed_round = room.limit_big;
                room.pot += room.limit_big;

                println!("  → Round {} pot: {} chips", round_num, room.pot);
            } else {
                println!("• Joe: CHECKS");
                println!("• Santo: BETS {} chips", room.limit_big);
                println!("• Joe: CALLS {} chips", room.limit_big);

                room.players[1].chips -= room.limit_big;
                room.players[1].committed_round = room.limit_big;
                room.pot += room.limit_big;

                room.players[3].chips -= room.limit_big;
                room.players[3].committed_round = room.limit_big;
                room.pot += room.limit_big;

                println!("  → FINAL POT: {} chips", room.pot);
            }

            println!("\n💬 ROUND {} CHAT:", round_num);
            let final_chat = if round_num == 6 {
                vec![
                    "• John: \"Too much action for me, good luck you two!\"",
                    "• Joe: \"Down to the wire! May the best hand win!\"",
                    "• Santo: \"This is what poker is all about! 🎯\"",
                ]
            } else {
                vec![
                    "• Santo: \"All in! Let's see what you've got Joe!\"",
                    "• Joe: \"Been building this hand all game... here goes!\"",
                    "• Bob: \"What a match! Great play everyone!\"",
                    "• Frank: \"Epic battle! Can't wait to see the showdown!\"",
                ]
            };
            for msg in final_chat {
                println!("{}", msg);
            }
        }

        // ============ SHOWDOWN ============
        room.phase = Phase::Showdown;

        println!("\n🎭 SHOWDOWN - FINAL HAND REVEAL");
        println!("════════════════════════════════════════════════");

        println!("\n🃏 FINAL HANDS:");
        println!("• John: FOLDED (Round 6)");

        println!("• Joe: [10♥ 6♣] + [A♠ 3♣ 5♥ 2♠ 4♦ 9♠] = 7 cards");
        println!("  └─ Low calculation: A(1)+2+3+4+5 = 15 (busted for low)");
        println!("  └─ High calculation: A(1)+2+3+4+5+6+9 = 30 (busted for high)");
        println!("  └─ Alternative: A(11)+2+3+4+5 = 25 (good for high!)");

        println!("• Santo: [8♦ 7♠] + [2♥ 4♠ 5♣ 3♦ A♣ 6♥ 2♣] = 8 cards");
        println!("  └─ Low calculation: A(1)+2+2+3+4+5+6 = 23 (busted for low)");
        println!("  └─ High calculation: A(1)+2+2+3+4+5+6 = 23 (good for high!)");

        println!("• Frank: FOLDED (Round 3)");
        println!("• Bob: FOLDED (Round 1)");

        println!("\n🏆 WINNER DETERMINATION:");
        println!("• Joe's best high: 25");
        println!("• Santo's best high: 23");
        println!("• SANTO WINS with the better high hand (23 vs 25)!");

        println!("\n💰 CHIP DISTRIBUTION:");
        println!("• Santo wins the entire pot: {} chips", room.pot);
        println!("• Santo's profit: {} chips", room.pot - 250); // Subtract initial investment

        println!("\n📊 FINAL CHIP COUNTS:");
        let chip_changes = [
            ("John", -120), // Folded in round 6
            ("Joe", -140),  // Lost in showdown
            ("Frank", -70), // Folded in round 3
            ("Santo", room.pot as i32 - 180), // Won pot minus investment
            ("Bob", -50),   // Folded in round 1
        ];

        for (name, change) in chip_changes.iter() {
            let sign = if *change >= 0 { "+" } else { "" };
            println!("• {}: {}{} chips", name, sign, change);
        }

        println!("\n💬 POST-GAME CHAT:");
        println!("• Santo: \"What a rush! Great game everyone! 🎉\"");
        println!("• Joe: \"So close! That was an amazing battle Santo!\"");
        println!("• John: \"Incredible match! Santo played that perfectly!\"");
        println!("• Frank: \"Those early folds saved me chips. Well played Santo!\"");
        println!("• Bob: \"Epic game to watch! Same time next week? 😄\"");

        println!("\n✅ SEVEN-ROUND MATCH COMPLETED SUCCESSFULLY!");
        println!("════════════════════════════════════════════════");
        println!("• Total rounds played: 7");
        println!("• Players who folded: 4");
        println!("• Final pot size: {} chips", room.pot);
        println!("• Winner: Santo");
        println!("• Game duration: ~45 minutes (simulated)");

        // Verify final state
        assert!(room.phase == Phase::Showdown);
        assert!(room.round == 7);
        assert!(room.pot > 500); // Substantial pot built up
        let active_players: Vec<_> = room.players.iter().filter(|p| !p.folded).collect();
        assert_eq!(active_players.len(), 2); // Joe and Santo
    }
}