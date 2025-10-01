use iced::{Element, Task, Length, Alignment};
use iced_widget::{button, column, container, row, text, Space, text_input, scrollable};
use futures::SinkExt;
use crate::messages::Msg;
use crate::App;

impl App {
    pub fn handle_lounge_msg(&mut self, msg: &Msg) -> Task<Msg> {
        match msg {
            Msg::JoinLounge => {
                // Connect to the lounge
                self.url = "ws://127.0.0.1:9001/ws".to_string();
                self.room = "lounge".to_string();
                self.in_lounge = true;
                self.app_state = crate::states::AppState::ConnectOverlay;
                self.connecting = true;
                self.log("🚪 Joining lounge...".to_string());
                Task::none()
            }
            Msg::LeaveLounge => {
                // Send leave message and reset state
                if let Some(ref tx) = self.tx_out {
                    let _ = tx.unbounded_send(cctmog_protocol::ClientToServer::LeaveLounge);
                }
                self.in_lounge = false;
                self.lounge_players.clear();
                self.chat_messages.clear(); // Clear chat messages to prevent tripling on rejoin
                self.connecting = false;
                self.connected = false;
                Task::none()
            }
            Msg::ChatInputChanged(input) => {
                self.chat_input = input.to_string();
                Task::none()
            }
            Msg::SendChat => {
                if !self.chat_input.trim().is_empty() && self.in_lounge {
                    if let Some(ref tx) = self.tx_out {
                        let _ = tx.unbounded_send(cctmog_protocol::ClientToServer::Chat {
                            message: self.chat_input.clone(),
                            scope: cctmog_protocol::MessageScope::Group, // Use Group scope for lounge
                        });
                    }
                    self.chat_input.clear();
                }
                Task::none()
            }
            Msg::CreateTable => {
                self.app_state = crate::states::AppState::TableCreation;
                Task::none()
            }
            Msg::HostGame => {
                // Legacy host game - redirect to lounge
                self.url = "ws://127.0.0.1:9001/ws".to_string();
                self.room = "lounge".to_string();
                self.in_lounge = true;
                self.app_state = crate::states::AppState::ConnectOverlay;
                self.connecting = true;
                self.log("🚪 Redirecting to lounge to host...".to_string());
                Task::none()
            }
            Msg::BrowseTables => {
                self.app_state = crate::states::AppState::TableBrowser;
                Task::none()
            }
            Msg::JoinTable => {
                // Legacy join table - redirect to lounge
                self.url = "ws://127.0.0.1:9001/ws".to_string();
                self.room = "lounge".to_string();
                self.in_lounge = true;
                self.app_state = crate::states::AppState::ConnectOverlay;
                self.connecting = true;
                self.log("🚪 Redirecting to lounge to join...".to_string());
                Task::none()
            }
            Msg::BackToHome => {
                self.app_state = crate::states::AppState::NameInput;
                Task::none()
            }
            Msg::HostInputChanged(input) => {
                self.host_input = input.to_string();
                Task::none()
            }
            Msg::ConnectToHost => {
                // Parse the input as "hostname:port" or just "port"
                if let Some((name, port_str)) = self.host_input.split_once(':') {
                    if let Ok(port) = port_str.parse::<u16>() {
                        self.host_name = Some(name.to_string());
                        self.host_server_port = Some(port);
                        self.log(format!("✓ Host set to: {} on port {}", name, port));
                    } else {
                        self.log("⚠️ Invalid port number".to_string());
                    }
                } else if let Ok(port) = self.host_input.parse::<u16>() {
                    self.host_name = Some("localhost".to_string());
                    self.host_server_port = Some(port);
                    self.log(format!("✓ Host set to: localhost on port {}", port));
                } else {
                    self.log("⚠️ Please enter host:port (e.g., localhost:9002) or just a port number".to_string());
                }
                Task::none()
            }
            Msg::VolunteerToHost => {
                // Start embedded server and volunteer as host
                // Using a default port of 9002 (can be made configurable later)
                let port = 9002;
                self.is_hosting = true;
                self.host_server_port = Some(port);
                self.log(format!("🎯 Starting embedded server on port {}...", port));

                // Return a task to start the embedded server
                Task::perform(
                    async move {
                        use tokio::net::TcpListener;
                        // Check if port is available
                        if TcpListener::bind(format!("127.0.0.1:{}", port)).await.is_ok() {
                            Ok(port)
                        } else {
                            Err(format!("Port {} is already in use", port))
                        }
                    },
                    |result| match result {
                        Ok(port) => Msg::EmbeddedServerStarted(port),
                        Err(err) => Msg::EmbeddedServerError(err),
                    },
                )
            }
            Msg::SelectHost(player_name, port) => {
                // Check if already selected - if so, deselect
                if let Some((selected_name, selected_port)) = &self.my_selected_host {
                    if selected_name == player_name && selected_port == port {
                        // Already selected - deselect
                        if let Some(ref tx) = self.tx_out {
                            let _ = tx.unbounded_send(cctmog_protocol::ClientToServer::SelectHost {
                                host_name: String::new(), // Empty string = deselect
                                port: 0,
                            });
                            self.log(format!("❌ Deselected {} as host", player_name));
                        }
                        self.my_selected_host = None;
                        return Task::none();
                    }
                }

                // Not selected or different selection - send selection to server
                if let Some(ref tx) = self.tx_out {
                    let _ = tx.unbounded_send(cctmog_protocol::ClientToServer::SelectHost {
                        host_name: player_name.clone(),
                        port: *port,
                    });
                    self.log(format!("✅ Selected {} as host. Waiting for everyone to select...", player_name));
                }
                self.my_selected_host = Some((player_name.clone(), *port));
                Task::none()
            }
            Msg::ConnectToOwnServer(port) => {
                // Connect to our own embedded server after the delay
                // Leave the lounge
                if let Some(ref tx) = self.tx_out {
                    let _ = tx.unbounded_send(cctmog_protocol::ClientToServer::LeaveLounge);
                }
                self.in_lounge = false;

                // Transition to connecting state, then will go to dealer selection
                self.app_state = crate::states::AppState::ConnectOverlay;
                self.connecting = true;
                self.connected = false;
                self.tx_out = None;  // Reset connection to reconnect to our own server

                self.log(format!("🎮 Connecting to your own game server on port {}...", port));
                Task::none()
            }
            Msg::SelectPlayerToHost(player_name) => {
                // A player name was clicked
                // Check if this player is hosting
                if let Some((_, port)) = self.available_hosts.iter().find(|(name, _)| name == player_name) {
                    // They're hosting - select/deselect them
                    return self.handle_lounge_msg(&Msg::SelectHost(player_name.clone(), *port));
                } else if *player_name == self.name {
                    // User clicked themselves and they're NOT hosting - volunteer to host
                    return self.handle_lounge_msg(&Msg::VolunteerToHost);
                } else {
                    // Clicked someone else who's not hosting - show error
                    self.log(format!("⚠️ {} is not hosting. They must volunteer to host first.", player_name));
                    Task::none()
                }
            }
            _ => Task::none(),
        }
    }

    pub fn lounge_view(&self) -> Element<'_, Msg> {
        // Check if we're in the lounge and showing room status
        if self.in_lounge {
            return self.lounge_room_view();
        }

        // Auto-connect to lounge if not already connected
        if !self.connecting && !self.connected && !self.in_lounge {
            return self.auto_join_lounge_view();
        }

        // Show the lounge interface directly
        self.lounge_room_view()
    }

    pub fn auto_join_lounge_view(&self) -> Element<'_, Msg> {
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
                text(format!("Hello, {}! Connecting to the shared room...", self.name))
                    .size(18)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                        ..Default::default()
                    })
            )
            .center_x(Length::Fill),

            Space::with_height(Length::Fixed(40.0)),

            // Auto-join lounge button
            container(
                button(
                    container(
                        column![
                            text("🚪 Enter Lounge").size(24),
                            Space::with_height(Length::Fixed(8.0)),
                            text("Join the main lounge where everyone gathers").size(16)
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
                .on_press(Msg::JoinLounge)
                .width(Length::Fixed(500.0))
                .height(Length::Fixed(100.0))
                .style(|theme: &iced::Theme, status| {
                    iced_widget::button::Style {
                        background: Some(iced::Background::Color(match status {
                            iced_widget::button::Status::Hovered => iced::Color::from_rgb(0.4, 0.6, 0.4),
                            _ => iced::Color::from_rgb(0.3, 0.5, 0.3),
                        })),
                        text_color: iced::Color::from_rgb(0.95, 0.95, 0.95),
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.5, 0.7, 0.5),
                            width: 3.0,
                            radius: iced::border::Radius::from(15.0),
                        },
                        ..Default::default()
                    }
                })
            )
            .center_x(Length::Fill),

            Space::with_height(Length::Fixed(60.0)),

            // Select Host button
            button(text("🔍 Select Host"))
                .on_press(Msg::CheckForHost)
                .padding(12)
                .width(Length::Fixed(180.0))
                .style(|_theme: &iced::Theme, status| iced_widget::button::Style {
                    background: Some(iced::Background::Color(match status {
                        iced_widget::button::Status::Hovered => iced::Color::from_rgb(0.4, 0.5, 0.7),
                        _ => iced::Color::from_rgb(0.3, 0.4, 0.6),
                    })),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.5, 0.6, 0.8),
                        width: 2.0,
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

    pub fn lounge_room_view(&self) -> Element<'_, Msg> {
        // Logo in top left (10% of height)
        let logo = container(
            text("CCTMOG")
                .size(48)
                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.95, 0.89, 0.78)), // #f3e3c7
                    ..Default::default()
                })
        )
        .padding(20);

        // Chat panel on the left
        let chat_panel = self.lounge_chat_view();

        // Right panel with title, players, and leave button
        let right_panel = column![
            Space::with_height(Length::Fixed(20.0)),

            // "Poker Lounge" title
            container(
                text("Poker Lounge")
                    .size(32)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.95, 0.89, 0.78)), // #f3e3c7
                        ..Default::default()
                    })
            )
            .center_x(Length::Fill),

            Space::with_height(Length::Fixed(40.0)),

            // Players section
            container(
                column![
                    text("PLAYERS IN LOUNGE")
                        .size(16)
                        .style(|_theme: &iced::Theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.84, 0.95, 0.95)), // #d5f2f3
                            ..Default::default()
                        }),
                    Space::with_height(Length::Fixed(20.0)),
                    column(
                        self.lounge_players
                            .iter()
                            .map(|player_name| {
                                let is_you = player_name == &self.name;
                                let is_hosting = self.available_hosts.iter().any(|(name, _)| name == player_name);
                                let is_selected = self.my_selected_host.as_ref()
                                    .map(|(name, _)| name == player_name)
                                    .unwrap_or(false);

                                // Find what host this player selected
                                let their_selection = self.player_selections.iter()
                                    .find(|(name, _)| name == player_name)
                                    .and_then(|(_, sel)| sel.as_ref());

                                let display_name = if is_you && is_hosting && is_selected {
                                    format!("✅ 🎯 {} (You - Hosting - Selected)", player_name)
                                } else if is_you && is_hosting {
                                    format!("🎯 {} (You - Hosting)", player_name)
                                } else if is_you {
                                    if let Some(selected_host) = their_selection {
                                        format!("{} (You) → {}", player_name, selected_host)
                                    } else {
                                        format!("{} (You)", player_name)
                                    }
                                } else if is_hosting && is_selected {
                                    format!("✅ 🎯 {} (Hosting - Selected)", player_name)
                                } else if is_hosting {
                                    if let Some(selected_host) = their_selection {
                                        format!("🎯 {} (Hosting) → {}", player_name, selected_host)
                                    } else {
                                        format!("🎯 {} (Hosting)", player_name)
                                    }
                                } else {
                                    if let Some(selected_host) = their_selection {
                                        format!("{} → {}", player_name, selected_host)
                                    } else {
                                        player_name.clone()
                                    }
                                };

                                button(
                                    text(display_name)
                                        .size(18)
                                )
                                .on_press(Msg::SelectPlayerToHost(player_name.clone()))
                                .padding(12)
                                .width(Length::Fixed(240.0))
                                .style(move |_theme: &iced::Theme, status| iced_widget::button::Style {
                                    background: Some(iced::Background::Color(match status {
                                        iced_widget::button::Status::Hovered if is_hosting => iced::Color::from_rgb(0.3, 0.7, 0.3),
                                        iced_widget::button::Status::Hovered if is_you => iced::Color::from_rgb(0.79, 0.64, 0.51),
                                        iced_widget::button::Status::Hovered => iced::Color::from_rgb(0.74, 0.59, 0.46),
                                        _ if is_hosting => iced::Color::from_rgb(0.2, 0.6, 0.2),
                                        _ if is_you => iced::Color::from_rgb(0.69, 0.54, 0.41), // #b08968 - seat color
                                        _ => iced::Color::from_rgb(0.64, 0.49, 0.36),
                                    })),
                                    text_color: if is_hosting {
                                        iced::Color::WHITE
                                    } else {
                                        iced::Color::from_rgb(0.12, 0.12, 0.12)
                                    },
                                    border: iced::Border {
                                        color: if is_selected {
                                            iced::Color::from_rgb(1.0, 0.84, 0.0) // Gold border when selected
                                        } else if is_hosting {
                                            iced::Color::from_rgb(0.3, 0.8, 0.3)
                                        } else {
                                            iced::Color::from_rgb(0.59, 0.44, 0.31)
                                        },
                                        width: if is_selected { 3.5 } else if is_hosting { 2.5 } else { 1.5 },
                                        radius: iced::border::Radius::from(18.0),
                                    },
                                    ..Default::default()
                                })
                                .into()
                            })
                            .collect::<Vec<_>>()
                    )
                    .spacing(12),
                ]
            )
            .center_x(Length::Fill),
        ]
        .align_x(Alignment::Center)
        .spacing(8)
        .width(Length::Fill);

        let main_content = row![
            chat_panel,
            Space::with_width(Length::Fixed(60.0)),
            right_panel,
        ]
        .align_y(Alignment::Start)
        .spacing(20);

        let full_layout = column![
            logo,
            main_content,
        ]
        .spacing(0);

        container(full_layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .style(|_theme: &iced::Theme| iced_widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(0.18, 0.14, 0.13))), // #2f2320 (dark brown)
                ..Default::default()
            })
            .into()
    }

    pub fn lounge_chat_view(&self) -> Element<'_, Msg> {
        let total_messages = self.chat_messages.len();

        // Build a vector of elements including date separators
        let mut elements: Vec<Element<'_, Msg>> = vec![];
        let mut last_date: Option<String> = None;

        for (idx, (player_name, message, _scope, timestamp)) in self.chat_messages.iter().enumerate() {
            // Parse timestamp to get date (format: "2025-09-30T23:38:56.256330+00:00")
            let current_date = timestamp.split('T').next().unwrap_or("");

            // Add date separator if day changed
            if last_date.as_deref() != Some(current_date) && !current_date.is_empty() {
                // Parse date to make it more readable (e.g., "September 30, 2025")
                let date_label = if let Some(parts) = current_date.split('-').collect::<Vec<_>>().get(0..3) {
                    let year = parts[0];
                    let month = match parts[1] {
                        "01" => "January", "02" => "February", "03" => "March", "04" => "April",
                        "05" => "May", "06" => "June", "07" => "July", "08" => "August",
                        "09" => "September", "10" => "October", "11" => "November", "12" => "December",
                        _ => parts[1]
                    };
                    let day = parts[2].trim_start_matches('0');
                    format!("{} {}, {}", month, day, year)
                } else {
                    current_date.to_string()
                };

                elements.push(
                    container(
                        text(date_label)
                            .size(14)
                            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.60, 0.60, 0.60)),
                                ..Default::default()
                            })
                    )
                    .width(Length::Fill)
                    .padding(8)
                    .center_x(Length::Fill)
                    .style(|_theme: &iced::Theme| iced_widget::container::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.08, 0.18, 0.19))),
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.15, 0.25, 0.26),
                            width: 1.0,
                            radius: iced::border::Radius::from(6.0),
                        },
                        ..Default::default()
                    })
                    .into()
                );
                last_date = Some(current_date.to_string());
            }

            // Parse time from timestamp (HH:MM)
            let time_str = if let Some(time_part) = timestamp.split('T').nth(1) {
                let time_parts: Vec<&str> = time_part.split(':').collect();
                if time_parts.len() >= 2 {
                    format!("{}:{}", time_parts[0], time_parts[1])
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            // Highlight the MOST RECENT message (the last one)
            let is_newest = idx == total_messages.saturating_sub(1);

            elements.push(
                container(
                    column![
                        row![
                            text(format!("{}: ", player_name))
                                .size(20)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.56, 0.89, 0.94)), // #8ee4f0
                                    ..Default::default()
                                }),
                            text(message)
                                .size(20)
                                .style(move |_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(
                                        if is_newest {
                                            iced::Color::from_rgb(1.0, 1.0, 1.0)  // Pure white for newest
                                        } else {
                                            iced::Color::from_rgb(0.80, 0.91, 0.91)  // #cde7e8
                                        }
                                    ),
                                    ..Default::default()
                                }),
                        ]
                        .spacing(4),
                        if !time_str.is_empty() {
                            Element::from(text(time_str)
                                .size(12)
                                .style(|_theme: &iced::Theme| iced_widget::text::Style {
                                    color: Some(iced::Color::from_rgb(0.50, 0.60, 0.62)),
                                    ..Default::default()
                                }))
                        } else {
                            Element::from(Space::with_height(Length::Fixed(0.0)))
                        }
                    ]
                    .spacing(4)
                )
                .padding(16)
                .width(Length::Fill)
                .style(move |_theme: &iced::Theme| iced_widget::container::Style {
                    background: Some(iced::Background::Color(
                        if is_newest {
                            iced::Color::from_rgb(0.20, 0.45, 0.50)  // Very bright teal for newest message
                        } else {
                            iced::Color::from_rgb(0.10, 0.23, 0.24)  // Normal bubble color
                        }
                    )),
                    border: iced::Border {
                        color: if is_newest {
                            iced::Color::from_rgb(0.30, 0.75, 0.85)  // Very bright cyan border for newest
                        } else {
                            iced::Color::from_rgb(0.05, 0.29, 0.31)  // Normal border
                        },
                        width: if is_newest { 3.0 } else { 1.5 },
                        radius: iced::border::Radius::from(12.0),
                    },
                    ..Default::default()
                })
                .into()
            );
        }

        let chat_messages = scrollable(column(elements).spacing(12))
            .height(Length::Fixed(400.0))  // Reduced from 550 to make room for input
            .direction(iced_widget::scrollable::Direction::Vertical(
                iced_widget::scrollable::Scrollbar::default()
                    .anchor(iced_widget::scrollable::Anchor::End)  // Auto-scroll to bottom
            ));

        let chat_input = row![
            text_input("💬 Type a message…", &self.chat_input)
                .on_input(Msg::ChatInputChanged)
                .on_submit(Msg::SendChat)
                .width(Length::Fill)
                .padding(18)
                .size(22)  // Larger text size
                .style(|_theme: &iced::Theme, _status| iced_widget::text_input::Style {
                    background: iced::Background::Color(iced::Color::from_rgb(0.15, 0.35, 0.38)), // Much brighter teal background
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.30, 0.70, 0.75), // Very bright border
                        width: 3.0,  // Thicker border
                        radius: iced::border::Radius::from(12.0),
                    },
                    icon: iced::Color::from_rgb(0.95, 0.95, 0.95),
                    placeholder: iced::Color::from_rgb(0.95, 0.95, 0.95).scale_alpha(0.7),
                    value: iced::Color::from_rgb(1.0, 1.0, 1.0),  // Pure white text
                    selection: iced::Color::from_rgb(0.30, 0.50, 0.55),
                }),
            Space::with_width(Length::Fixed(12.0)),
            button(
                text("Send")
                    .size(20)
            )
                .on_press(Msg::SendChat)
                .padding(18)
                .style(|_theme: &iced::Theme, status| iced_widget::button::Style {
                    background: Some(iced::Background::Color(match status {
                        iced_widget::button::Status::Hovered => iced::Color::from_rgb(0.35, 0.75, 0.80),
                        _ => iced::Color::from_rgb(0.25, 0.65, 0.70),
                    })),
                    text_color: iced::Color::from_rgb(1.0, 1.0, 1.0),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.35, 0.75, 0.80),
                        width: 3.0,
                        radius: iced::border::Radius::from(12.0),
                    },
                    ..Default::default()
                }),
        ]
        .align_y(Alignment::Center);

        container(
            column![
                // Title
                text("LOBBY CHAT")
                    .size(28)
                    .style(|_theme: &iced::Theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.84, 0.95, 0.95)), // #d5f2f3
                        ..Default::default()
                    }),

                Space::with_height(Length::Fixed(15.0)),

                // Chat messages
                chat_messages,

                Space::with_height(Length::Fixed(15.0)),

                // Input bar - MUST BE VISIBLE!
                chat_input,
            ]
        )
        .width(Length::Fixed(460.0))
        .height(Length::Fixed(600.0))  // Reduced from 780 to ensure it fits on screen
        .padding(25)  // Reduced from 30 to save space
        .style(|_theme: &iced::Theme| iced_widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.06, 0.17, 0.18))), // #0f2b2e - panel color
            border: iced::Border {
                color: iced::Color::from_rgb(0.03, 0.19, 0.20), // #073034
                width: 2.0,
                radius: iced::border::Radius::from(16.0),
            },
            ..Default::default()
        })
        .into()
    }
}