use iced::{Element, Task};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn handle_table_choice_msg(&mut self, msg: &Msg) -> Task<Msg> {
        match msg {
            Msg::CreateTable => {
                self.app_state = crate::states::AppState::TableCreation;
                Task::none()
            }
            Msg::BrowseTables => {
                self.app_state = crate::states::AppState::TableBrowser;
                self.send_message(cctmog_protocol::ClientToServer::ListTables);
                Task::none()
            }
            Msg::JoinTable => {
                self.app_state = crate::states::AppState::ConnectOverlay;
                self.connecting = true;
                Task::none()
            }
            Msg::BackToHome => {
                self.app_state = crate::states::AppState::NameInput;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn table_choice_view(&self) -> Element<'_, Msg> {
        crate::ui::views::table_choice_view(self)
    }
}