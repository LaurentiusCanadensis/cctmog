pub mod seven_twenty_seven;
pub mod omaha;
pub mod texas_holdem;

use cctmog_protocol::{GameVariant, Card, PublicRoom, PrivateHand};
use iced::Element;
use crate::messages::Msg;
use crate::App;

/// Trait defining common game behavior
pub trait GameLogic {
    /// Get the display name for this game variant
    fn name(&self) -> &'static str;

    /// Get the game variant enum
    fn variant(&self) -> GameVariant;

    /// Render game-specific UI elements
    fn render_game_ui(&self, room: &PublicRoom, hand: &PrivateHand) -> Element<'static, Msg>;

    /// Handle game-specific actions
    fn handle_game_action(&self, app: &mut App, msg: &Msg);

    /// Get available actions for the current game state
    fn available_actions(&self, room: &PublicRoom, hand: &PrivateHand, is_your_turn: bool) -> Vec<Element<'static, Msg>>;

    /// Validate if an action is legal in the current game state
    fn is_action_valid(&self, room: &PublicRoom, hand: &PrivateHand, action: &str) -> bool;
}

/// Factory function to get game logic implementation
pub fn get_game_logic(variant: GameVariant) -> Box<dyn GameLogic> {
    match variant {
        GameVariant::SevenTwentySeven => Box::new(seven_twenty_seven::SevenTwentySevenGame),
        GameVariant::Omaha => Box::new(omaha::OmahaGame),
        GameVariant::TexasHoldem => Box::new(texas_holdem::TexasHoldemGame),
    }
}