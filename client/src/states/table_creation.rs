use iced::{Element, Task};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn handle_table_creation_msg(&mut self, msg: &Msg) -> Task<Msg> {
        match msg {
            Msg::TableNameChanged(name) => {
                self.table_name = name.clone();
                self.table_creation_error = None;
                Task::none()
            }
            Msg::TableGameVariantChanged(variant) => {
                self.table_game_variant = *variant;
                Task::none()
            }
            Msg::TableAnteChanged(ante) => {
                self.table_ante = ante.clone();
                Task::none()
            }
            Msg::TableLimitSmallChanged(limit) => {
                self.table_limit_small = limit.clone();
                Task::none()
            }
            Msg::TableLimitBigChanged(limit) => {
                self.table_limit_big = limit.clone();
                Task::none()
            }
            Msg::TableMaxRaisesChanged(raises) => {
                self.table_max_raises = raises.clone();
                Task::none()
            }
            Msg::SubmitTableCreation => {
                // Validate inputs
                if self.table_name.trim().is_empty() {
                    self.table_creation_error = Some("Table name is required".to_string());
                    return Task::none();
                }

                let ante = self.table_ante.parse::<u64>().unwrap_or(0);
                let limit_small = self.table_limit_small.parse::<u64>().unwrap_or(0);
                let limit_big = self.table_limit_big.parse::<u64>().unwrap_or(0);
                let max_raises = self.table_max_raises.parse::<u32>().unwrap_or(0);

                if ante == 0 {
                    self.table_creation_error = Some("Ante must be a positive number".to_string());
                    return Task::none();
                }

                self.table_creation_error = None;
                self.app_state = crate::states::AppState::ConnectOverlay;
                self.connecting = true;

                let create_msg = cctmog_protocol::ClientToServer::CreateTable {
                    name: self.table_name.clone(),
                    game_variant: self.table_game_variant,
                    ante,
                    limit_small,
                    limit_big,
                    max_raises,
                };

                self.pending_table_creation = Some(create_msg);
                Task::none()
            }
            Msg::StartEmbeddedServerForTable => {
                self.app_state = crate::states::AppState::ConnectOverlay;
                self.connecting = true;

                let server = crate::embedded_server::EmbeddedServer::new(self.local_server_port);
                self.embedded_server = Some(server);

                Task::perform(
                    async move { 8080 }, // placeholder port
                    Msg::EmbeddedServerStarted
                )
            }
            Msg::BackToHome => {
                self.app_state = crate::states::AppState::TableChoice;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn table_creation_view(&self) -> Element<'_, Msg> {
        crate::ui::views::table_creation_view(self)
    }
}