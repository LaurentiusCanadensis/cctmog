use crate::messages::Msg;
use iced::Subscription;

pub fn websocket_subscription(_url: String, _room: String, _name: String) -> Subscription<Msg> {
    Subscription::none()
}
