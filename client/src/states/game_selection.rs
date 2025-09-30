use iced::{Element, Length, Alignment};
use iced_widget::{button, column, container, text, Space};
use cctmog_protocol::GameVariant;
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn game_selection_view(&self) -> Element<'_, Msg> {
        let content = column![
            Space::with_height(Length::Fixed(40.0)),
            container(
                text("Select Game")
                    .size(32)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                        ..Default::default()
                    })
            )
            .center_x(Length::Fill),

            Space::with_height(Length::Fixed(20.0)),

            container(
                text("Choose which poker variant to play:")
                    .size(18)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                        ..Default::default()
                    })
            )
            .center_x(Length::Fill),

            Space::with_height(Length::Fixed(40.0)),

            // Game variant buttons
            column![
                // Seven Twenty Seven
                button(
                    container(
                        column![
                            text("7/27 (Seven Twenty-Seven)")
                                .size(24)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                                    ..Default::default()
                                }),
                            Space::with_height(Length::Fixed(8.0)),
                            text("A unique poker variant where you aim for 7 or 27 points")
                                .size(14)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                    ..Default::default()
                                }),
                        ]
                        .align_x(Alignment::Center)
                        .spacing(4)
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .padding(30)
                )
                .on_press(Msg::GameVariantChosen(GameVariant::SevenTwentySeven))
                .width(Length::Fixed(500.0))
                .height(Length::Fixed(100.0))
                .style(|_theme: &iced::Theme, status| iced_widget::button::Style {
                    background: Some(iced::Background::Color(match status {
                        iced_widget::button::Status::Hovered => iced::Color::from_rgb(0.3, 0.3, 0.35),
                        _ => iced::Color::from_rgb(0.2, 0.2, 0.25),
                    })),
                    text_color: iced::Color::from_rgb(0.9, 0.9, 0.9),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.36, 0.62, 0.98),
                        width: 2.0,
                        radius: iced::border::Radius::from(12.0),
                    },
                    ..Default::default()
                }),

                Space::with_height(Length::Fixed(20.0)),

                // Texas Hold'em
                button(
                    container(
                        column![
                            text("Texas Hold'em")
                                .size(24)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                                    ..Default::default()
                                }),
                            Space::with_height(Length::Fixed(8.0)),
                            text("The most popular poker game in the world")
                                .size(14)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                    ..Default::default()
                                }),
                        ]
                        .align_x(Alignment::Center)
                        .spacing(4)
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .padding(30)
                )
                .on_press(Msg::GameVariantChosen(GameVariant::TexasHoldem))
                .width(Length::Fixed(500.0))
                .height(Length::Fixed(100.0))
                .style(|_theme: &iced::Theme, status| iced_widget::button::Style {
                    background: Some(iced::Background::Color(match status {
                        iced_widget::button::Status::Hovered => iced::Color::from_rgb(0.3, 0.3, 0.35),
                        _ => iced::Color::from_rgb(0.2, 0.2, 0.25),
                    })),
                    text_color: iced::Color::from_rgb(0.9, 0.9, 0.9),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.36, 0.62, 0.98),
                        width: 2.0,
                        radius: iced::border::Radius::from(12.0),
                    },
                    ..Default::default()
                }),

                Space::with_height(Length::Fixed(20.0)),

                // Omaha
                button(
                    container(
                        column![
                            text("Omaha")
                                .size(24)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                                    ..Default::default()
                                }),
                            Space::with_height(Length::Fixed(8.0)),
                            text("Four hole cards, must use exactly two in your hand")
                                .size(14)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                    ..Default::default()
                                }),
                        ]
                        .align_x(Alignment::Center)
                        .spacing(4)
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .padding(30)
                )
                .on_press(Msg::GameVariantChosen(GameVariant::Omaha))
                .width(Length::Fixed(500.0))
                .height(Length::Fixed(100.0))
                .style(|_theme: &iced::Theme, status| iced_widget::button::Style {
                    background: Some(iced::Background::Color(match status {
                        iced_widget::button::Status::Hovered => iced::Color::from_rgb(0.3, 0.3, 0.35),
                        _ => iced::Color::from_rgb(0.2, 0.2, 0.25),
                    })),
                    text_color: iced::Color::from_rgb(0.9, 0.9, 0.9),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.36, 0.62, 0.98),
                        width: 2.0,
                        radius: iced::border::Radius::from(12.0),
                    },
                    ..Default::default()
                }),
            ]
            .spacing(15)
            .align_x(Alignment::Center),

            Space::with_height(Length::Fixed(40.0)),

            // Back button
            button(
                text("Back to Dealer Selection")
                    .size(16)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                        ..Default::default()
                    })
            )
            .on_press(Msg::BackToHome)
            .padding(12)
            .width(Length::Fixed(200.0))
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