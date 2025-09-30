use iced::{Element, Length};
use iced_widget::{container, Image};
use iced_widget::image::Handle as ImageHandle;
use crate::messages::Msg;

pub fn cctmog_logo() -> Element<'static, Msg> {
    // put the PNG at client/assets/cctmog_logo.png
    Image::new(ImageHandle::from_path("client/assets/cctmog_logo.png"))
        .width(Length::Fixed(220.0))
        .into()
}