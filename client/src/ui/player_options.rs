use iced::{Alignment, Element, Length};
use iced_widget::{button, column, container, row, text, Space};

use uuid::Uuid;
use cctmog_protocol::{PublicRoom, Phase};

use crate::messages::Msg;
use crate::ui::cards::{cards_row_svg, CardSize};

pub fn player_options_view(
    room: &PublicRoom,
    your_id: Option<Uuid>,
    your_seat: Option<usize>,
    your_hand: &cctmog_protocol::PrivateHand,
) -> Element<'static, Msg> {
    let your_player = room.players.iter().find(|p| {
        your_id.map(|id| id == p.id).unwrap_or(false)
            || your_seat.map(|seat| seat == p.seat).unwrap_or(false)
    });

    let your_cards_section = if let Some(player) = your_player {
        if !your_hand.down_cards.is_empty() {
            column![
                text("Your Cards")
                    .size(14)
                    .style(|_theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
                        ..Default::default()
                    }),
                Space::with_height(Length::Fixed(8.0)),
                cards_row_svg(&your_hand.down_cards, CardSize::Large, 8.0),
                if !player.up_cards.is_empty() {
                    column![
                        Space::with_height(Length::Fixed(8.0)),
                        text("Up Cards")
                            .size(12)
                            .style(|_theme| iced_widget::text::Style {
                                color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
                                ..Default::default()
                            }),
                        Space::with_height(Length::Fixed(4.0)),
                        cards_row_svg(&player.up_cards, CardSize::Medium, 4.0),
                    ]
                } else {
                    column![]
                }
            ]
        } else {
            column![
                text("Waiting for cards...")
                    .size(14)
                    .style(|_theme| iced_widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.6, 0.6, 0.6)),
                        ..Default::default()
                    })
            ]
        }
    } else {
        column![
            text("Not seated at table")
                .size(14)
                .style(|_theme| iced_widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.8, 0.4, 0.4)),
                    ..Default::default()
                })
        ]
    };

    let action_buttons = if let Some(player) = your_player {
        let is_your_turn = room.to_act_seat == player.seat && room.phase == Phase::Acting;

        if is_your_turn {
            // Get game-specific actions
            let game_logic = crate::games::get_game_logic(room.game_variant);
            let actions = game_logic.available_actions(room, your_hand, is_your_turn);

            if actions.is_empty() {
                row![
                    text("No actions available")
                        .size(12)
                        .style(|_theme| iced_widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                            ..Default::default()
                        })
                ]
            } else {
                let mut action_row = row![].spacing(8);
                for action in actions {
                    action_row = action_row.push(action);
                }
                action_row
            }
        } else {
            row![
                text(if room.phase == Phase::Lobby {
                    "Waiting for game to start..."
                } else {
                    "Waiting for your turn..."
                })
                .size(12)
                .style(|_theme| iced_widget::text::Style {
                    color: Some(iced::Color::from_rgb(0.7, 0.7, 0.7)),
                    ..Default::default()
                })
            ]
        }
    } else {
        row![
            button(text("Join Table").size(12))
                .on_press(Msg::JoinTable)
                .style(|_theme: &iced::Theme, _status| iced_widget::button::Style {
                    background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.7, 0.3))),
                    text_color: iced::Color::WHITE,
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.1, 0.5, 0.2),
                        width: 1.0,
                        radius: iced::border::Radius::from(4.0),
                    },
                    ..Default::default()
                })
                .width(Length::Fixed(120.0)),
        ]
    };

    // Add game-specific UI
    let game_ui = if let Some(_) = your_player {
        let game_logic = crate::games::get_game_logic(room.game_variant);
        game_logic.render_game_ui(room, your_hand)
    } else {
        Space::with_height(Length::Fixed(0.0)).into()
    };

    container(
        column![
            your_cards_section,
            Space::with_height(Length::Fixed(8.0)),
            game_ui,
            Space::with_height(Length::Fill),
            action_buttons,
        ]
        .spacing(8)
        .align_x(Alignment::Center)
    )
    .width(Length::Fixed(60.0 * 10.0)) // 60 units wide
    .height(Length::Fixed(20.0 * 10.0)) // 20 units high
    .padding([12, 16])
    .style(|_theme| iced_widget::container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.05, 0.05, 0.05, 0.9))),
        border: iced::Border {
            color: iced::Color::from_rgb(0.3, 0.3, 0.3),
            width: 1.0,
            radius: iced::border::Radius::from(6.0),
        },
        ..Default::default()
    })
    .into()
}