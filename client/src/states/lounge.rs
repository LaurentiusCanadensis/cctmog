use iced::{Element, Task, Length, Alignment};
use iced_widget::{button, column, container, row, text, Space};
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn handle_lounge_msg(&mut self, msg: &Msg) -> Task<Msg> {
        // Check for available host when entering lounge (if not hosting and no host found yet)
        if !self.is_hosting && self.host_name.is_none() {
            self.check_for_available_host();
        }

        match msg {
            Msg::CreateTable => {
                self.app_state = crate::states::AppState::TableCreation;
                Task::none()
            }
            Msg::HostGame => {
                // Start embedded server and then go to dealer selection
                self.is_hosting = true;
                self.log("🎯 Starting server to host game...".to_string());
                Task::perform(
                    async {
                        // Find an available port starting from 9001
                        use std::net::TcpListener;
                        for port in 9001..9100 {
                            if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
                                drop(listener);
                                return Ok(port);
                            }
                        }
                        Err("No available ports found".to_string())
                    },
                    |result| match result {
                        Ok(port) => Msg::EmbeddedServerStarted(port),
                        Err(err) => Msg::EmbeddedServerError(err),
                    },
                )
            }
            Msg::BrowseTables => {
                self.app_state = crate::states::AppState::TableBrowser;
                Task::none()
            }
            Msg::JoinTable => {
                // Check if there's a host before allowing join
                if let Some(host_name) = &self.host_name {
                    if let Some(port) = self.host_server_port {
                        // Join the host's server
                        self.url = format!("ws://127.0.0.1:{}/ws", port);
                        self.room = "shared_game_room".to_string();
                        self.app_state = crate::states::AppState::ConnectOverlay;
                        self.connecting = true;
                        self.log(format!("🎮 Joining {}'s game on port {}", host_name, port));
                    } else {
                        self.log("❌ Host found but no server port available".to_string());
                    }
                } else {
                    self.log("❌ No host available. Someone must host a game first!".to_string());
                }
                Task::none()
            }
            Msg::BackToHome => {
                self.app_state = crate::states::AppState::NameInput;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn lounge_view(&self) -> Element<'_, Msg> {
        // Create 6 menu options in a 60x60 grid layout
        // Each option is 60 length x 10 width
        let option_width = Length::Fixed(480.0); // 60 units * 8px
        let option_height = Length::Fixed(80.0);  // 10 units * 8px

        let option_style = |theme: &iced::Theme, status| {
            iced_widget::button::Style {
                background: Some(iced::Background::Color(match status {
                    iced_widget::button::Status::Hovered => iced::Color::from_rgb(0.3, 0.3, 0.35),
                    _ => iced::Color::from_rgb(0.2, 0.2, 0.25),
                })),
                text_color: iced::Color::from_rgb(0.9, 0.9, 0.9),
                border: iced::Border {
                    color: iced::Color::from_rgb(0.4, 0.4, 0.4),
                    width: 2.0,
                    radius: iced::border::Radius::from(12.0),
                },
                ..Default::default()
            }
        };

        let content = column![
            Space::with_height(Length::Fixed(40.0)),
            container(
                text("Welcome to the Lounge")
                    .size(32)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.92, 0.92, 0.94)),
                        ..Default::default()
                    })
            )
            .center_x(Length::Fill),

            Space::with_height(Length::Fixed(20.0)),

            container(
                text(format!("Hello, {}! Choose your adventure:", self.name))
                    .size(18)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                        ..Default::default()
                    })
            )
            .center_x(Length::Fill),

            Space::with_height(Length::Fixed(40.0)),

            // First row of options
            row![
                button(
                    container(
                        column![
                            text("🎮 Quick Game").size(20),
                            Space::with_height(Length::Fixed(8.0)),
                            text("Join the shared poker room").size(14)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                    ..Default::default()
                                })
                        ]
                        .align_x(Alignment::Center)
                        .spacing(4)
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                )
                .on_press(Msg::JoinTable)
                .width(option_width)
                .height(option_height)
                .style(option_style),

                Space::with_width(Length::Fixed(20.0)),

                button(
                    container(
                        column![
                            text("🎯 Host Game").size(20),
                            Space::with_height(Length::Fixed(8.0)),
                            text("Host a game for your friends").size(14)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                    ..Default::default()
                                })
                        ]
                        .align_x(Alignment::Center)
                        .spacing(4)
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                )
                .on_press(Msg::HostGame)
                .width(option_width)
                .height(option_height)
                .style(option_style),
            ]
            .spacing(20)
            .align_y(Alignment::Center),

            Space::with_height(Length::Fixed(20.0)),

            // Second row of options
            row![
                button(
                    container(
                        column![
                            text("🔍 Browse Tables").size(20),
                            Space::with_height(Length::Fixed(8.0)),
                            text("Find and join existing games").size(14)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                    ..Default::default()
                                })
                        ]
                        .align_x(Alignment::Center)
                        .spacing(4)
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                )
                .on_press(Msg::BrowseTables)
                .width(option_width)
                .height(option_height)
                .style(option_style),

                Space::with_width(Length::Fixed(20.0)),

                button(
                    container(
                        column![
                            text("📊 Statistics").size(20),
                            Space::with_height(Length::Fixed(8.0)),
                            text("View your game history").size(14)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                    ..Default::default()
                                })
                        ]
                        .align_x(Alignment::Center)
                        .spacing(4)
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                )
                .on_press(Msg::ViewStats)
                .width(option_width)
                .height(option_height)
                .style(option_style),
            ]
            .spacing(20)
            .align_y(Alignment::Center),

            Space::with_height(Length::Fixed(20.0)),

            // Third row of options
            row![
                button(
                    container(
                        column![
                            text("⚙️ Settings").size(20),
                            Space::with_height(Length::Fixed(8.0)),
                            text("Configure your preferences").size(14)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                    ..Default::default()
                                })
                        ]
                        .align_x(Alignment::Center)
                        .spacing(4)
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                )
                .on_press(Msg::OpenSettings)
                .width(option_width)
                .height(option_height)
                .style(option_style),

                Space::with_width(Length::Fixed(20.0)),

                button(
                    container(
                        column![
                            text("📖 Tutorial").size(20),
                            Space::with_height(Length::Fixed(8.0)),
                            text("Learn how to play").size(14)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                                    ..Default::default()
                                })
                        ]
                        .align_x(Alignment::Center)
                        .spacing(4)
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                )
                .on_press(Msg::OpenTutorial)
                .width(option_width)
                .height(option_height)
                .style(option_style),
            ]
            .spacing(20)
            .align_y(Alignment::Center),

            Space::with_height(Length::Fixed(40.0)),

            // Back button
            button(text("Back"))
                .on_press(Msg::BackToHome)
                .padding(12)
                .width(Length::Fixed(120.0))
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
            .into()
    }
}