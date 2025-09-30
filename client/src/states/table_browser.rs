use iced::{Element, Task};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn handle_table_browser_msg(&mut self, msg: &Msg) -> Task<Msg> {
        match msg {
            Msg::JoinTableByName(table_name) => {
                self.room = table_name.clone();
                self.app_state = crate::states::AppState::ConnectOverlay;
                self.connecting = true;
                Task::none()
            }
            Msg::BackToHome => {
                self.app_state = crate::states::AppState::TableChoice;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn table_browser_view(&self) -> Element<'_, Msg> {
        crate::ui::views::table_browser_view(self)
    }
}