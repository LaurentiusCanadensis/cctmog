use iced::{Element, Task};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn handle_name_input_msg(&mut self, msg: &Msg) -> Task<Msg> {
        match msg {
            Msg::NameChanged(name) => {
                self.name = name.clone();
                self.name_error = None;
                Task::none()
            }
            Msg::ConfirmName => {
                let trimmed_name = self.name.trim();
                if trimmed_name.is_empty() {
                    self.name_error = Some("Please enter a name".to_string());
                } else if trimmed_name.len() < 2 {
                    self.name_error = Some("Name must be at least 2 characters".to_string());
                } else if trimmed_name.len() > 20 {
                    self.name_error = Some("Name must be 20 characters or less".to_string());
                } else if !trimmed_name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                    self.name_error = Some("Name can only contain letters, numbers, _ and -".to_string());
                } else {
                    self.name_error = None;
                    // Go to the Lounge to choose what to do
                    self.app_state = crate::states::AppState::Lounge;
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn name_input_view(&self) -> Element<'_, Msg> {
        crate::ui::views::name_input_view(&self.name, &self.name_error, &self.available_usernames, &self.selected_username)
    }
}