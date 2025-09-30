use cctmog_protocol::{GameVariant, Card, PublicRoom, PrivateHand, Phase};
use iced::{Element, Length, Alignment};
use iced_widget::{button, column, container, row, text, Space};
use crate::messages::Msg;
use crate::App;
use crate::ui::cards::{cards_row_svg, CardSize};
use super::GameLogic;

pub struct OmahaGame;

impl GameLogic for OmahaGame {
    fn name(&self) -> &'static str {
        "Omaha"
    }

    fn variant(&self) -> GameVariant {
        GameVariant::Omaha
    }

    fn render_game_ui(&self, room: &PublicRoom, _hand: &PrivateHand) -> Element<'static, Msg> {
        let community_cards_display = if !room.community_cards.is_empty() {
            column![
                text("Community Cards:").size(14).style(|_theme| iced_widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                    ..Default::default()
                }),
                Space::with_height(Length::Fixed(4.0)),
                cards_row_svg(&room.community_cards, CardSize::Medium, 4.0),
            ].spacing(4)
        } else {
            column![
                text("Waiting for community cards...").size(12).style(|_theme| iced_widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    ..Default::default()
                })
            ]
        };

        let pot_info = text(format!("Pot: ${}", room.pot))
            .size(16)
            .style(|_theme| iced_widget::text::Style {
                color: Some(iced::Color::from_rgb(0.2, 0.8, 0.2)),
                ..Default::default()
            });

        let betting_info = if room.in_betting {
            column![
                text(format!("Current Bet: ${}", room.current_bet)).size(12),
                text(format!("Raises: {}/{}", room.raises_made, room.max_raises)).size(12),
            ].spacing(2)
        } else {
            column![]
        };

        let game_rules = column![
            text("Omaha Rules:").size(14).style(|_theme| iced_widget::text::Style {
                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.5)),
                ..Default::default()
            }),
            text("• Use exactly 2 hole cards + 3 community cards").size(10),
            text("• 4 hole cards dealt to each player").size(10),
            text("• 5 community cards (flop, turn, river)").size(10),
            text("• Multiple betting rounds").size(10),
        ].spacing(2);

        container(
            column![
                pot_info,
                Space::with_height(Length::Fixed(8.0)),
                betting_info,
                Space::with_height(Length::Fixed(12.0)),
                community_cards_display,
                Space::with_height(Length::Fixed(16.0)),
                game_rules,
            ]
            .spacing(8)
            .align_x(Alignment::Center)
        )
        .padding(12)
        .style(|_theme| iced_widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.1, 0.3, 0.8))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.3, 0.6),
                width: 1.0,
                radius: iced::border::Radius::from(6.0),
            },
            ..Default::default()
        })
        .into()
    }

    fn handle_game_action(&self, app: &mut App, msg: &Msg) {
        match msg {
            Msg::Fold => {
                app.send_message(cctmog_protocol::ClientToServer::Fold);
            }
            Msg::Check => {
                app.send_message(cctmog_protocol::ClientToServer::Check);
            }
            Msg::Bet => {
                app.send_message(cctmog_protocol::ClientToServer::Bet);
            }
            Msg::Call => {
                app.send_message(cctmog_protocol::ClientToServer::Call);
            }
            Msg::Raise => {
                app.send_message(cctmog_protocol::ClientToServer::Raise);
            }
            _ => {}
        }
    }

    fn available_actions(&self, room: &PublicRoom, _hand: &PrivateHand, is_your_turn: bool) -> Vec<Element<'static, Msg>> {
        if !is_your_turn || room.phase != Phase::Acting {
            return vec![];
        }

        let mut actions = vec![];

        // Always allow fold
        actions.push(
            button(text("Fold").size(12))
                .on_press(Msg::Fold)
                .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(0.8, 0.2, 0.2))),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.6, 0.1, 0.1),
                        width: 1.0,
                        radius: iced::border::Radius::from(4.0),
                    },
                    ..Default::default()
                })
                .width(Length::Fixed(80.0))
                .into()
        );

        if room.current_bet == 0 {
            // No bet to call, can check or bet
            actions.push(
                button(text("Check").size(12))
                    .on_press(Msg::Check)
                    .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                            width: 1.0,
                            radius: iced::border::Radius::from(4.0),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fixed(80.0))
                    .into()
            );

            actions.push(
                button(text("Bet").size(12))
                    .on_press(Msg::Bet)
                    .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.6, 0.8))),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.1, 0.4, 0.6),
                            width: 1.0,
                            radius: iced::border::Radius::from(4.0),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fixed(80.0))
                    .into()
            );
        } else {
            // There's a bet, can call or raise
            actions.push(
                button(text("Call").size(12))
                    .on_press(Msg::Call)
                    .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.6, 0.8))),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.1, 0.4, 0.6),
                            width: 1.0,
                            radius: iced::border::Radius::from(4.0),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fixed(80.0))
                    .into()
            );

            if room.raises_made < room.max_raises {
                actions.push(
                    button(text("Raise").size(12))
                        .on_press(Msg::Raise)
                        .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.8, 0.2))),
                            text_color: iced::Color::WHITE,
                            border: iced::Border {
                                color: iced::Color::from_rgb(0.1, 0.6, 0.1),
                                width: 1.0,
                                radius: iced::border::Radius::from(4.0),
                            },
                            ..Default::default()
                        })
                        .width(Length::Fixed(80.0))
                        .into()
                );
            }
        }

        actions
    }

    fn is_action_valid(&self, room: &PublicRoom, _hand: &PrivateHand, action: &str) -> bool {
        match action {
            "fold" => true,
            "check" => room.current_bet == 0,
            "bet" => room.current_bet == 0,
            "call" => room.current_bet > 0,
            "raise" => room.current_bet > 0 && room.raises_made < room.max_raises,
            _ => false,
        }
    }
}