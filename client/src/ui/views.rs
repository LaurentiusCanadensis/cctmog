// client/src/ui/views.rs
use iced::{Element, Length, Alignment};
use iced_widget::{button, column, container, row, text, text_input, Space, pick_list};

use cctmog_protocol::GameVariant;
use crate::messages::Msg;
use crate::app::App;
use crate::ui::shared::brand_logo;

pub fn splash_view() -> Element<'static, Msg> {
    container(
        column![
            Space::with_height(Length::Fixed(100.0)),
            brand_logo(),
            Space::with_height(Length::Fixed(50.0)),
            text("CCTMOG Poker").size(24),
            Space::with_height(Length::Fixed(20.0)),
            text("Loading...").size(16),
        ]
        .align_x(Alignment::Center)
        .spacing(10)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

pub fn name_input_view<'a>(
    name: &'a str,
    name_error: &'a Option<String>,
    available_usernames: &'a [String],
    selected_username: &'a Option<String>
) -> Element<'a, Msg> {
    let mut content = column![
        Space::with_height(Length::Fixed(100.0)),
        brand_logo(),
        Space::with_height(Length::Fixed(50.0)),
        text("Select Your Username").size(24),
        Space::with_height(Length::Fixed(20.0)),
        pick_list(
            available_usernames,
            selected_username.as_ref(),
            Msg::UsernameSelected
        )
        .width(Length::Fixed(300.0))
        .padding(10),
        Space::with_height(Length::Fixed(10.0)),
        text("Or enter a custom name:").size(16),
        Space::with_height(Length::Fixed(10.0)),
        text_input("Custom name", name)
            .on_input(Msg::NameChanged)
            .padding(10)
            .width(Length::Fixed(300.0)),
        Space::with_height(Length::Fixed(20.0)),
        button(text("Continue"))
            .on_press(Msg::ConfirmName)
            .padding(10)
            .width(Length::Fixed(150.0)),
    ]
    .align_x(Alignment::Center)
    .spacing(10);

    if let Some(error) = name_error {
        content = content.push(
            text(error)
                .style(|_theme| iced_widget::text::Style {
                    color: Some(iced::Color::from_rgb(1.0, 0.3, 0.3)),
                    ..Default::default()
                })
                .size(14)
        );
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn table_choice_view(app: &App) -> Element<'_, Msg> {
    container(
        column![
            Space::with_height(Length::Fixed(60.0)),
            brand_logo(),
            Space::with_height(Length::Fixed(40.0)),
            text(format!("Welcome, {}!", app.name)).size(24),
            Space::with_height(Length::Fixed(40.0)),

            // Action buttons
            column![
                button(text("Create Table").size(18))
                    .on_press(Msg::CreateTable)
                    .padding(15)
                    .width(Length::Fixed(200.0)),
                Space::with_height(Length::Fixed(20.0)),
                button(text("Browse Tables").size(18))
                    .on_press(Msg::BrowseTables)
                    .padding(15)
                    .width(Length::Fixed(200.0)),
            ]
            .align_x(Alignment::Center)
            .spacing(10)
        ]
        .align_x(Alignment::Center)
        .spacing(10)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

pub fn table_creation_view(app: &App) -> Element<'_, Msg> {
    let variant_options = vec![
        GameVariant::SevenTwentySeven,
        GameVariant::Omaha,
        GameVariant::TexasHoldem,
    ];

    let mut content = column![
        Space::with_height(Length::Fixed(40.0)),
        text("Create New Table").size(24),
        Space::with_height(Length::Fixed(30.0)),

        // Table name input
        row![
            text("Table Name:").width(Length::Fixed(120.0)),
            text_input("Enter table name", &app.table_name)
                .on_input(Msg::TableNameChanged)
                .padding(8)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),

        Space::with_height(Length::Fixed(15.0)),

        // Game variant selection
        row![
            text("Game Variant:").width(Length::Fixed(120.0)),
            pick_list(
                variant_options,
                Some(app.table_game_variant),
                Msg::TableGameVariantChanged
            )
            .placeholder("Select variant")
            .width(Length::Fixed(200.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),

        Space::with_height(Length::Fixed(15.0)),

        // Ante input
        row![
            text("Ante:").width(Length::Fixed(120.0)),
            text_input("10", &app.table_ante)
                .on_input(Msg::TableAnteChanged)
                .padding(8)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),

        Space::with_height(Length::Fixed(15.0)),

        // Small limit input
        row![
            text("Small Limit:").width(Length::Fixed(120.0)),
            text_input("10", &app.table_limit_small)
                .on_input(Msg::TableLimitSmallChanged)
                .padding(8)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),

        Space::with_height(Length::Fixed(15.0)),

        // Big limit input
        row![
            text("Big Limit:").width(Length::Fixed(120.0)),
            text_input("20", &app.table_limit_big)
                .on_input(Msg::TableLimitBigChanged)
                .padding(8)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),

        Space::with_height(Length::Fixed(15.0)),

        // Max raises input
        row![
            text("Max Raises:").width(Length::Fixed(120.0)),
            text_input("3", &app.table_max_raises)
                .on_input(Msg::TableMaxRaisesChanged)
                .padding(8)
                .width(Length::Fixed(200.0)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),

        Space::with_height(Length::Fixed(30.0)),

        // Action buttons
        row![
            button(text("Back"))
                .on_press(Msg::BackToHome)
                .padding(12)
                .width(Length::Fixed(100.0)),
            Space::with_width(Length::Fixed(20.0)),
            button(text("Create Table"))
                .on_press(Msg::SubmitTableCreation)
                .padding(12)
                .width(Length::Fixed(150.0)),
        ]
        .spacing(10),
    ]
    .align_x(Alignment::Center)
    .spacing(8);

    // Add error message if present
    if let Some(error) = &app.table_creation_error {
        content = content.push(
            Space::with_height(Length::Fixed(20.0))
        ).push(
            text(error)
                .style(|_theme| iced_widget::text::Style {
                    color: Some(iced::Color::from_rgb(1.0, 0.3, 0.3)),
                    ..Default::default()
                })
                .size(14)
        );
    }

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

pub fn table_browser_view(app: &App) -> Element<'_, Msg> {
    let mut table_list = column![]
        .spacing(10)
        .padding(20);

    if app.available_tables.is_empty() {
        table_list = table_list.push(text("No tables available").size(16));
    } else {
        for table in &app.available_tables {
            let table_info = format!(
                "{} - {} ({} players) - {}",
                table.name,
                table.game_variant,
                table.player_count,
                match table.phase {
                    cctmog_protocol::Phase::Lobby => "Waiting",
                    _ => "In Game",
                }
            );

            table_list = table_list.push(
                button(text(table_info))
                    .on_press(Msg::JoinTableByName(table.name.clone()))
                    .width(Length::Fill)
                    .padding(10),
            );
        }
    }

    container(
        column![
            Space::with_height(Length::Fixed(40.0)),
            text("Available Tables").size(24),
            Space::with_height(Length::Fixed(20.0)),
            container(table_list)
                .width(Length::Fixed(600.0))
                .height(Length::Fixed(400.0)),
            Space::with_height(Length::Fixed(20.0)),
            button(text("Back to Menu"))
                .on_press(Msg::BackToHome)
                .padding(10)
                .width(Length::Fixed(150.0)),
        ]
        .align_x(Alignment::Center)
        .spacing(10)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

pub fn connect_overlay(url: &str, name: &str, room: &str) -> Element<'static, Msg> {
    container(
        column![
            Space::with_height(Length::Fixed(100.0)),
            brand_logo(),
            Space::with_height(Length::Fixed(50.0)),
            text("Connecting...").size(24),
            Space::with_height(Length::Fixed(20.0)),
            text(format!("Connecting to {}", url)).size(16),
            text(format!("Player: {}", name)).size(14),
            text(format!("Room: {}", room)).size(14),
        ]
        .align_x(Alignment::Center)
        .spacing(10)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

pub fn game_view(app: &App) -> Element<'_, Msg> {
    use iced_widget::{scrollable, text_input};

    // Left panel: Game State Debug View
    let game_state_text = if let Some(ref snapshot) = app.snapshot {
        format!(
            "GAME STATE\n\
            ══════════════════════════════\n\
            Room: {}\n\
            Phase: {:?}\n\
            Variant: {}\n\
            ──────────────────────────────\n\
            Players: {}\n\
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
            snapshot.players.len(),
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
        )
    } else {
        "No game state available".to_string()
    };

    let left_panel = container(
        column![
            // Game state section
            container(
                scrollable(
                    text(game_state_text)
                        .size(12)
                        .font(iced::Font::MONOSPACE)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.8, 0.95, 0.8)),
                            ..Default::default()
                        })
                )
                .height(Length::Fixed(400.0))
            )
            .padding(10)
            .style(|_theme: &iced::Theme| iced_widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.05, 0.1, 0.05))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.2, 0.4, 0.2),
                    width: 1.0,
                    radius: iced::border::Radius::from(8.0),
                },
                ..Default::default()
            }),

            Space::with_height(Length::Fixed(10.0)),

            // Game chat section
            container(
                column![
                    text("GAME CHAT")
                        .size(14)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.95, 0.95, 0.8)),
                            ..Default::default()
                        }),
                    Space::with_height(Length::Fixed(5.0)),
                    scrollable(
                        column(
                            app.chat_messages
                                .iter()
                                .map(|(player_name, message, _scope, _timestamp)| {
                                    container(
                                        text(format!("{}: {}", player_name, message))
                                            .size(11)
                                            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                                                ..Default::default()
                                            })
                                    )
                                    .padding(4)
                                    .into()
                                })
                                .collect::<Vec<_>>()
                        )
                        .spacing(2)
                    )
                    .height(Length::Fixed(200.0)),
                    Space::with_height(Length::Fixed(5.0)),
                    row![
                        text_input("Type message...", &app.chat_input)
                            .on_input(Msg::ChatInputChanged)
                            .on_submit(Msg::SendChat)
                            .padding(6)
                            .size(12),
                        Space::with_width(Length::Fixed(5.0)),
                        button(text("Send").size(12))
                            .on_press(Msg::SendChat)
                            .padding(6),
                    ]
                ]
            )
            .padding(10)
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
    .width(Length::Fixed(350.0))
    .height(Length::Fill)
    .padding(10);

    // Main content area (right side)
    let mut main_content = column![
        Space::with_height(Length::Fixed(20.0)),
        text("Game Lobby").size(32)
            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                ..Default::default()
            }),
        Space::with_height(Length::Fixed(20.0)),
        {
            let status_text = if app.is_hosting {
                if app.waiting_for_players {
                    "⏳ Waiting for more players to join... (Ready when you are!)"
                } else {
                    "🎮 Ready to start! Waiting for your command..."
                }
            } else {
                "Waiting for players to join..."
            };

            text(status_text)
                .size(18)
                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                    ..Default::default()
                })
        },
        Space::with_height(Length::Fixed(40.0)),
    ]
    .align_x(Alignment::Center);

    // Add host controls if this user is the host
    if app.is_hosting {
        main_content = main_content.push(
            container(
                column![
                    {
                        let title = if app.waiting_for_players {
                            "👑 Host Controls (⏳ Waiting Mode)"
                        } else {
                            "👑 Host Controls"
                        };
                        text(title)
                            .size(20)
                            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                color: Some(iced::Color::from_rgb(1.0, 0.84, 0.0)),
                                ..Default::default()
                            })
                    },
                    Space::with_height(Length::Fixed(20.0)),
                    row![
                        button(
                            container(
                                text("🚀 Start Game Now")
                                    .size(16)
                            )
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                        )
                        .on_press(Msg::StartGameNow)
                        .width(Length::Fixed(200.0))
                        .height(Length::Fixed(50.0))
                        .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.7, 0.2))),
                            text_color: iced::Color::WHITE,
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.1, 0.5, 0.1),
                                width: 2.0,
                                radius: iced::border::Radius::from(8.0),
                            },
                            ..Default::default()
                        }),
                        Space::with_width(Length::Fixed(20.0)),
                        button(
                            container(
                                text("⏳ Wait for More")
                                    .size(16)
                            )
                            .center_x(Length::Fill)
                            .center_y(Length::Fill)
                        )
                        .on_press(Msg::WaitForMorePlayers)
                        .width(Length::Fixed(200.0))
                        .height(Length::Fixed(50.0))
                        .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(0.7, 0.7, 0.2))),
                            text_color: iced::Color::WHITE,
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.5, 0.5, 0.1),
                                width: 2.0,
                                radius: iced::border::Radius::from(8.0),
                            },
                            ..Default::default()
                        }),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                    Space::with_height(Length::Fixed(20.0)),
                    text("As the host, you can start the game when ready or wait for more players to join.")
                        .size(14)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                            ..Default::default()
                        })
                ]
                .align_x(Alignment::Center)
                .spacing(5)
            )
            .padding(20)
            .style(|_theme: &iced::Theme| iced_widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.15, 0.15, 0.18))),
                border: iced::Border {
                    color: iced::Color::from_rgb(1.0, 0.84, 0.0),
                    width: 2.0,
                    radius: iced::border::Radius::from(12.0),
                },
                ..Default::default()
            })
        );
    } else {
        main_content = main_content.push(
            container(
                text("🎮 Waiting for host to start the game...")
                    .size(16)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                        ..Default::default()
                    })
            )
            .padding(20)
            .style(|_theme: &iced::Theme| iced_widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.12, 0.12, 0.15))),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                    width: 1.0,
                    radius: iced::border::Radius::from(8.0),
                },
                ..Default::default()
            })
        );
    }

    // Combine left panel and main content
    let layout = row![
        left_panel,
        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    ];

    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_theme: &iced::Theme| iced_widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.10, 0.10, 0.12))),
            ..Default::default()
        })
        .into()
}

pub fn comments_view(app: &App) -> Element<'_, Msg> {
    let mut content = column![
        Space::with_height(Length::Fixed(40.0)),
        text("Game Complete - Share Your Thoughts").size(24),
        Space::with_height(Length::Fixed(30.0)),
    ].align_x(Alignment::Center);

    // Display existing comments
    if !app.game_comments.is_empty() {
        content = content.push(text("Comments:").size(18));
        content = content.push(Space::with_height(Length::Fixed(15.0)));

        for comment in &app.game_comments {
            content = content.push(
                container(
                    column![
                        text(format!("{}: {}", comment.player_name, comment.message))
                            .size(14),
                        text(&comment.timestamp)
                            .size(12)
                            .style(|_theme| iced_widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                                ..Default::default()
                            }),
                    ]
                    .spacing(4)
                )
                .padding(10)
                .style(|_theme| iced_widget::container::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(0.08, 0.08, 0.09))),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.25, 0.25, 0.25),
                        width: 1.0,
                        radius: iced::border::Radius::from(8.0),
                    },
                    ..Default::default()
                })
                .width(Length::Fixed(500.0))
            );
            content = content.push(Space::with_height(Length::Fixed(8.0)));
        }
    } else {
        content = content.push(
            text("No comments yet. Be the first to share your thoughts!")
                .size(16)
                .style(|_theme| iced_widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    ..Default::default()
                })
        );
    }

    content = content.push(Space::with_height(Length::Fixed(30.0)));

    // Comment input section
    content = content.push(
        column![
            text("Add your comment:").size(16),
            Space::with_height(Length::Fixed(10.0)),
            text_input("Share your thoughts about this game...", &app.comment_input)
                .on_input(Msg::CommentInputChanged)
                .on_submit(Msg::PostComment)
                .padding(12)
                .width(Length::Fixed(500.0)),
            Space::with_height(Length::Fixed(15.0)),
            row![
                button(text("Post Comment"))
                    .on_press(Msg::PostComment)
                    .padding(12)
                    .width(Length::Fixed(150.0))
                    .style(|_theme, _status| iced_widget::button::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.36, 0.62, 0.98))),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.30, 0.56, 0.92),
                            width: 2.0,
                            radius: iced::border::Radius::from(8.0),
                        },
                        ..Default::default()
                    }),
                Space::with_width(Length::Fixed(20.0)),
                button(text(if app.ready_to_continue { "Ready ✓" } else { "Continue to Next Game" }))
                    .on_press(Msg::ContinueToNextGame)
                    .padding(12)
                    .width(Length::Fixed(200.0))
                    .style(move |_theme, _status| iced_widget::button::Style {
                        background: Some(iced::Background::Color(if app.ready_to_continue {
                            iced::Color::from_rgb(0.2, 0.7, 0.2)
                        } else {
                            iced::Color::from_rgb(0.2, 0.6, 0.2)
                        })),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            color: if app.ready_to_continue {
                                iced::Color::from_rgb(0.1, 0.6, 0.1)
                            } else {
                                iced::Color::from_rgb(0.1, 0.5, 0.1)
                            },
                            width: 2.0,
                            radius: iced::border::Radius::from(8.0),
                        },
                        ..Default::default()
                    }),
            ]
            .spacing(10),
        ]
        .align_x(Alignment::Center)
        .spacing(8)
    );

    content = content.push(Space::with_height(Length::Fixed(30.0)));

    // Back button
    content = content.push(
        button(text("Back to Home"))
            .on_press(Msg::BackToHome)
            .padding(10)
            .width(Length::Fixed(150.0))
            .style(|_theme, _status| iced_widget::button::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                text_color: iced::Color::WHITE,
                border: iced::Border {
                    color: iced::Color::from_rgb(0.4, 0.4, 0.4),
                    width: 1.0,
                    radius: iced::border::Radius::from(8.0),
                },
                ..Default::default()
            })
    );

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}