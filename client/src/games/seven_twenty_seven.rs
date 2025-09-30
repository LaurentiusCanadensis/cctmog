use cctmog_protocol::{GameVariant, Card, PublicRoom, PrivateHand, Phase};
use iced::{Element, Length, Alignment};
use iced_widget::{button, column, container, row, text, Space};
use crate::messages::Msg;
use crate::App;
use crate::ui::cards::{cards_row_svg, CardSize};
use super::GameLogic;

pub struct SevenTwentySevenGame;

impl GameLogic for SevenTwentySevenGame {
    fn name(&self) -> &'static str {
        "7/27"
    }

    fn variant(&self) -> GameVariant {
        GameVariant::SevenTwentySeven
    }

    fn render_game_ui(&self, _room: &PublicRoom, hand: &PrivateHand) -> Element<'static, Msg> {

        let score_info = if !hand.down_cards.is_empty() {
            let score = cctmog_protocol::score_hand(&hand.down_cards);
            column![
                text("Your Score:").size(14),
                if let Some(score_7) = score.best_under_7 {
                    text(format!("Best under 7: {:.1}", score_7)).size(12)
                } else {
                    text("No valid score under 7").size(12)
                },
                if let Some(score_27) = score.best_under_27 {
                    text(format!("Best under 27: {:.1}", score_27)).size(12)
                } else {
                    text("Busted! (over 27)").size(12).style(|_theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.9, 0.3, 0.3)),
                        ..Default::default()
                    })
                },
            ].spacing(4)
        } else {
            column![]
        };

        let game_rules = column![
            text("7/27 Rules:").size(14).style(|_theme| iced_widget::text::Style {
                color: Some(iced::Color::from_rgb(0.9, 0.9, 0.5)),
                ..Default::default()
            }),
            text("• Get as close to 7 or 27 as possible").size(10),
            text("• Aces: 1 or 11 points").size(10),
            text("• Face cards: 0.5 points").size(10),
            text("• Number cards: face value").size(10),
            text("• Take cards or stand").size(10),
        ].spacing(2);

        container(
            column![
                score_info,
                Space::with_height(Length::Fixed(16.0)),
                game_rules,
            ]
            .spacing(8)
            .align_x(Alignment::Center)
        )
        .padding(12)
        .style(|_theme| iced_widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.3, 0.1, 0.8))),
            border: iced::Border {
                color: iced::Color::from_rgb(0.3, 0.6, 0.3),
                width: 1.0,
                radius: iced::border::Radius::from(6.0),
            },
            ..Default::default()
        })
        .into()
    }

    fn handle_game_action(&self, app: &mut App, msg: &Msg) {
        match msg {
            Msg::TakeCard => {
                app.send_message(cctmog_protocol::ClientToServer::TakeCard);
            }
            Msg::Stand => {
                app.send_message(cctmog_protocol::ClientToServer::Stand);
            }
            _ => {}
        }
    }

    fn available_actions(&self, room: &PublicRoom, hand: &PrivateHand, is_your_turn: bool) -> Vec<Element<'static, Msg>> {
        if !is_your_turn || room.phase != Phase::Acting {
            return vec![];
        }

        let can_take_card = hand.down_cards.len() < 7; // Max 7 cards in 7/27

        let mut actions = vec![];

        if can_take_card {
            actions.push(
                button(text("Take Card").size(12))
                    .on_press(Msg::TakeCard)
                    .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                        background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.7, 0.2))),
                        text_color: iced::Color::WHITE,
                        border: iced::Border {
                            color: iced::Color::from_rgb(0.1, 0.5, 0.1),
                            width: 1.0,
                            radius: iced::border::Radius::from(4.0),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fixed(100.0))
                    .into()
            );
        }

        actions.push(
            button(text("Stand").size(12))
                .on_press(Msg::Stand)
                .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(0.6, 0.4, 0.2))),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.4, 0.2, 0.1),
                        width: 1.0,
                        radius: iced::border::Radius::from(4.0),
                    },
                    ..Default::default()
                })
                .width(Length::Fixed(100.0))
                .into()
        );

        actions
    }

    fn is_action_valid(&self, _room: &PublicRoom, hand: &PrivateHand, action: &str) -> bool {
        match action {
            "take_card" => hand.down_cards.len() < 7,
            "stand" => true,
            _ => false,
        }
    }
}