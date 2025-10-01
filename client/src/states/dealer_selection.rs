use iced::{Element, Length, Alignment};
use iced_widget::{button, column, container, text, Space, row, scrollable};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn dealer_selection_view(&self) -> Element<'_, Msg> {
        let (players, game_debug_text) = if let Some(ref snapshot) = self.snapshot {
            let debug_text = format!(
                "GAME STATE (Room struct)\n\
                ══════════════════════════════\n\
                Room: {}\n\
                Phase: {:?}\n\
                Variant: {}\n\
                ──────────────────────────────\n\
                Players:\n{}\n\
                ──────────────────────────────\n\
                Dealer Seat: {}\n\
                To Act: {}\n\
                ──────────────────────────────\n\
                Pot: ${}\n\
                Current Bet: ${}\n\
                Raises: {}/{}\n\
                Round: {}\n\
                In Betting: {}\n\
                ──────────────────────────────\n\
                Ante: ${}\n\
                Limits: ${} / ${}\n\
                Community Cards: {}\n\
                ──────────────────────────────\n\
                Elected Players: {}\n\
                Dealer ID: {:?}\n\
                ══════════════════════════════",
                snapshot.room,
                snapshot.phase,
                snapshot.game_variant,
                snapshot.players.iter().enumerate()
                    .map(|(i, p)| format!("  [{}] {} - ${} chips, {} cards",
                        i, p.name, p.chips, p.cards_count))
                    .collect::<Vec<_>>()
                    .join("\n"),
                snapshot.dealer_seat,
                snapshot.to_act_seat,
                snapshot.pot,
                snapshot.current_bet,
                snapshot.raises_made,
                snapshot.max_raises,
                snapshot.round,
                snapshot.in_betting,
                snapshot.ante,
                snapshot.limit_small,
                snapshot.limit_big,
                snapshot.community_cards.len(),
                snapshot.elected_players.len(),
                snapshot.current_dealer_id,
            );
            (&snapshot.players, debug_text)
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

        // Left/center content - dealer selection
        let main_content = column![
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

        // Right panel - Game struct display
        let right_panel = container(
            column![
                // Game state debug display
                container(
                    scrollable(
                        text(game_debug_text)
                            .size(11)
                            .font(iced::Font::MONOSPACE)
                            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.8, 0.95, 0.8)),
                                ..Default::default()
                            })
                    )
                    .height(Length::Fixed(500.0))
                )
                .padding(15)
                .style(|_theme: &iced::Theme| iced_widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(0.05, 0.1, 0.05))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.2, 0.4, 0.2),
                        width: 2.0,
                        radius: iced::border::Radius::from(8.0),
                    },
                    ..Default::default()
                }),

                Space::with_height(Length::Fixed(15.0)),

                // Your hand display
                container(
                    column![
                        text("YOUR HAND")
                            .size(14)
                            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.95, 0.95, 0.8)),
                                ..Default::default()
                            }),
                        Space::with_height(Length::Fixed(8.0)),
                        text(if self.your_hand.down_cards.is_empty() {
                            "No cards yet".to_string()
                        } else {
                            format!("Cards: {}",
                                self.your_hand.down_cards.iter()
                                    .map(|c| c.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )
                        })
                            .size(12)
                            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                                ..Default::default()
                            })
                    ]
                )
                .padding(12)
                .style(|_theme: &iced::Theme| iced_widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(0.08, 0.08, 0.12))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.3, 0.3, 0.4),
                        width: 1.0,
                        radius: iced::border::Radius::from(8.0),
                    },
                    ..Default::default()
                }),
            ]
        )
        .width(Length::Fixed(400.0))
        .height(Length::Fill)
        .padding(10);

        // Combine main content and right panel
        let layout = row![
            container(main_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
            right_panel,
        ];

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme: &iced::Theme| iced_widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.10, 0.10, 0.11))),
                ..Default::default()
            })
            .into()
    }
}