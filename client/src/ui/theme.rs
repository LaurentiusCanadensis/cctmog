use iced::{Color, Border};
use iced::border::Radius;

pub const FELT: Color        = Color { r: 0.070, g: 0.345, b: 0.190, a: 1.0 };
pub const INK_SOFT: Color    = Color { r: 0.14,  g: 0.14,  b: 0.16,  a: 1.0 };
pub const TEXT: Color        = Color { r: 0.92,  g: 0.92,  b: 0.94,  a: 1.0 };

pub fn plate() -> iced_widget::container::Style {
    iced_widget::container::Style {
        background: Some(iced::Background::Color(INK_SOFT)),
        border: Border {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.50),
            width: 1.0,
            radius: Radius::from(10.0),
        },
        text_color: Some(TEXT),
        ..Default::default()
    }
}