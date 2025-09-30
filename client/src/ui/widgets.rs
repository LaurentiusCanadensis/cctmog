use crate::messages::Msg;
use iced::{Element, Length};

pub fn pill(label: String) -> Element<'static, Msg> {
    iced::widget::container(iced::widget::text(label).size(16))
        .padding([6, 10])
        .width(Length::Shrink)
        .into()
}
