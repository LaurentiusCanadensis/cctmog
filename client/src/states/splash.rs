use iced::{Element, Task};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn handle_splash_msg(&mut self, msg: &Msg) -> Task<Msg> {
        match msg {
            Msg::SplashFinished => {
                self.app_state = crate::states::AppState::NameInput;
                Task::none()
            }
            Msg::Tick => {
                if let Some(start_time) = self.splash_start_time {
                    if start_time.elapsed().as_secs() >= 2 {
                        self.app_state = crate::states::AppState::NameInput;
                        self.splash_start_time = None;
                    }
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn splash_view(&self) -> Element<'_, Msg> {
        crate::ui::views::splash_view()
    }
}