use iced::{Element, Length};
use iced_widget::{column, container, text, Space};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn dealer_splash_view(&self) -> Element<'_, Msg> {
        let fallback = "Someone".to_string();
        let dealer_name = self.selected_dealer.as_ref()
            .unwrap_or(&fallback);

        container(
            column![
                Space::with_height(Length::Fill),
                container(
                    text(format!("{} is Dealer", dealer_name))
                        .size(48)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.98, 0.86, 0.22)), // GOLD color
                            ..Default::default()
                        })
                )
                .center_x(Length::Fill),
                Space::with_height(Length::Fixed(20.0)),
                container(
                    text("🎰")
                        .size(64)
                )
                .center_x(Length::Fill),
                Space::with_height(Length::Fill),
            ]
            .align_x(iced::Alignment::Center)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(|_theme: &iced::Theme| iced_widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.10, 0.10, 0.11))),
            ..Default::default()
        })
        .into()
    }
}