// client/src/ui/mod.rs
pub mod actions;
pub mod table;
pub mod widgets;
mod logo;
pub(crate) mod ws;
pub mod cards;
pub mod canvas;
mod theme;
pub mod shared;
pub mod views;
pub mod header;
pub mod game_table;
pub mod player_options;
pub mod state;


use iced::{Color, Border};
use iced::border::Radius;
use iced_widget::container;

pub const FELT: Color        = Color { r: 0.070, g: 0.345, b: 0.190, a: 1.0 };
pub const FELT_DARK: Color   = Color { r: 0.045, g: 0.220, b: 0.120, a: 1.0 };
pub const LIP: Color         = Color { r: 0.020, g: 0.090, b: 0.050, a: 1.0 };
pub const GOLD: Color        = Color { r: 0.980, g: 0.860, b: 0.220, a: 1.0 };
pub const TEXT: Color        = Color { r: 0.92,  g: 0.92,  b: 0.94,  a: 1.0 };
pub const INK_SOFT: Color    = Color { r: 0.14,  g: 0.14,  b: 0.16,  a: 1.0 };

pub fn plate() -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(INK_SOFT)),
        border: Border { color: Color::from_rgba(0.0, 0.0, 0.0, 0.50), width: 1.0, radius: Radius::from(10.0) },
        text_color: Some(TEXT),
        ..Default::default()
    }
}

// handy pill
pub fn pill(label: String) -> iced::Element<'static, crate::messages::Msg> {
    use iced::{Length};
    use iced_widget::{container, text};
    container(text(label).size(16))
        .padding(8.0_f32)
        .width(Length::Shrink)
        .style(|_| plate())
        .into()
}