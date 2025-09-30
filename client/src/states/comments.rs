use iced::{Element, Task};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn handle_comments_msg(&mut self, msg: &Msg) -> Task<Msg> {
        match msg {
            Msg::CommentInputChanged(input) => {
                self.comment_input = input.clone();
                Task::none()
            }
            Msg::PostComment => {
                if !self.comment_input.trim().is_empty() {
                    self.send_message(cctmog_protocol::ClientToServer::PostComment {
                        message: self.comment_input.clone(),
                    });
                    self.comment_input.clear();
                }
                Task::none()
            }
            Msg::ContinueToNextGame => {
                self.send_message(cctmog_protocol::ClientToServer::ContinueToNextGame);
                self.ready_to_continue = true;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn comments_view(&self) -> Element<'_, Msg> {
        crate::ui::views::comments_view(self)
    }
}