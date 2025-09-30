use iced::{Element, Task};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn handle_connect_overlay_msg(&mut self, msg: &Msg) -> Task<Msg> {
        match msg {
            Msg::WsConnected(tx) => {
                self.tx_out = Some(tx.clone());
                self.connected = true;
                self.connecting = false;

                // Send pending table creation if exists
                if let Some(create_msg) = self.pending_table_creation.take() {
                    self.send_message(create_msg);
                } else {
                    // Join existing room
                    self.send_message(cctmog_protocol::ClientToServer::Join {
                        room: self.room.clone(),
                        name: self.name.clone(),
                    });
                }
                Task::none()
            }
            Msg::WsError(error) => {
                self.log.push(format!("WS Error: {}", error));
                self.connecting = false;
                self.connected = false;
                Task::none()
            }
            Msg::BackToHome => {
                self.app_state = crate::states::AppState::TableChoice;
                self.connecting = false;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn connect_overlay_view(&self) -> Element<'_, Msg> {
        crate::ui::views::connect_overlay(&self.url, &self.name, &self.room)
    }
}