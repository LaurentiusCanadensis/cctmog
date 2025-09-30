use iced::{Element, Length, Alignment};
use iced_widget::{button, column, container, text, Space};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn dealer_selection_view(&self) -> Element<'_, Msg> {
        let players = if let Some(ref snapshot) = self.snapshot {
            &snapshot.players
        } else {
            // Fallback for when no game snapshot is available
            return container(
                text("Loading players...")
                    .size(24)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                        ..Default::default()
                    })
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        };

        let content = column![
            Space::with_height(Length::Fixed(40.0)),
            container(
                text("Select Dealer")
                    .size(32)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                        ..Default::default()
                    })
            )
            .center_x(Length::Fill),

            Space::with_height(Length::Fixed(20.0)),

            container(
                text("Choose who will be the dealer for this game:")
                    .size(18)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                        ..Default::default()
                    })
            )
            .center_x(Length::Fill),

            Space::with_height(Length::Fixed(40.0)),

            // Player buttons
            column(
                players.iter().map(|player| {
                    let is_you = self.your_id.map(|id| id == player.id).unwrap_or(false);
                    let display_name = if is_you {
                        format!("{} (You)", player.name)
                    } else {
                        player.name.clone()
                    };

                    button(
                        container(
                            text(display_name)
                                .size(20)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                                    ..Default::default()
                                })
                        )
                        .center_x(Length::Fill)
                        .center_y(Length::Fill)
                        .padding(20)
                    )
                    .on_press(Msg::DealerSelected(player.name.clone()))
                    .width(Length::Fixed(300.0))
                    .height(Length::Fixed(60.0))
                    .style(|_theme: &iced::Theme, status| iced_widget::button::Style {
                        background: Some(iced::Background::Color(match status {
                            iced_widget::button::Status::Hovered => iced::Color::from_rgb(0.46, 0.72, 1.0),
                            _ => iced::Color::from_rgb(0.36, 0.62, 0.98),
                        })),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.56, 0.82, 1.0),
                            width: 2.0,
                            radius: iced::border::Radius::from(8.0),
                        },
                        ..Default::default()
                    })
                    .into()
                }).collect::<Vec<_>>()
            )
            .spacing(15)
            .align_x(Alignment::Center),

            Space::with_height(Length::Fixed(40.0)),

            // Back button
            button(
                text("Back to Game")
                    .size(16)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                        ..Default::default()
                    })
            )
            .on_press(Msg::BackToHome)
            .padding(12)
            .width(Length::Fixed(150.0))
            .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    color: iced::Color::from_rgb(0.4, 0.4, 0.4),
                    width: 1.0,
                    radius: iced::border::Radius::from(8.0),
                },
                ..Default::default()
            }),
        ]
        .align_x(Alignment::Center)
        .spacing(10);

        container(content)
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