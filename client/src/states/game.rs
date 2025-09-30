use iced::{Element, Task};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn handle_game_msg(&mut self, msg: &Msg) -> Task<Msg> {
        // First check if this is a game-specific action
        if let Some(ref snapshot) = self.snapshot {
            let game_logic = crate::games::get_game_logic(snapshot.game_variant);
            game_logic.handle_game_action(self, msg);
        }

        // Handle general game messages
        match msg {
            Msg::SitReady => {
                self.send_message(cctmog_protocol::ClientToServer::SitReady);
                Task::none()
            }
            Msg::StartHand => {
                self.send_message(cctmog_protocol::ClientToServer::StartHand);
                Task::none()
            }
            Msg::SelectGameVariant(variant) => {
                self.send_message(cctmog_protocol::ClientToServer::SelectGameVariant { variant: *variant });
                Task::none()
            }
            Msg::ChatInputChanged(input) => {
                self.chat_input = input.clone();
                Task::none()
            }
            Msg::SendChat => {
                if !self.chat_input.trim().is_empty() {
                    self.send_message(cctmog_protocol::ClientToServer::Chat {
                        message: self.chat_input.clone(),
                        scope: self.chat_scope,
                    });
                    self.chat_input.clear();
                }
                Task::none()
            }
            Msg::ToggleAssetTest => {
                self.show_asset_test = !self.show_asset_test;
                Task::none()
            }
            Msg::StartGameNow => {
                if self.is_hosting {
                    self.log("🎮 Host starting game now!".to_string());
                    self.waiting_for_players = false;
                    self.send_message(cctmog_protocol::ClientToServer::StartHand);
                    // Broadcast to all connected players that game is starting
                    self.log("📢 Broadcasting game start to all players".to_string());
                } else {
                    self.log("❌ Only the host can start the game".to_string());
                }
                Task::none()
            }
            Msg::WaitForMorePlayers => {
                if self.is_hosting {
                    self.waiting_for_players = true;
                    self.log("⏳ Host waiting for more players to join...".to_string());
                    self.log("🔄 Game will not start until host clicks 'Start Game Now'".to_string());
                } else {
                    self.log("❌ Only the host can control game timing".to_string());
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn game_view(&self) -> Element<'_, Msg> {
        crate::ui::views::game_view(self)
    }
}