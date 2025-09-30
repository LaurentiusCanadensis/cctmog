use iced::Length;
use iced_widget::{column, row, button, text, Space};
use cctmog_protocol::PublicRoom;
use crate::messages::Msg;
use cctmog_protocol::Phase;

/// Create a styled poker action button
fn poker_button(label: String, color: iced::Color, msg: Option<Msg>) -> iced::widget::Button<'static, Msg> {
    let mut btn = button(
        text(label)
            .size(16)
            .style(move |_theme| iced_widget::text::Style {
                color: Some(iced::Color::WHITE),
                ..Default::default()
            })
    )
    .padding([12, 20])
    .width(Length::Fixed(100.0))
    .style(move |_theme, status| {
        let (bg_color, border_color) = match status {
            iced_widget::button::Status::Hovered => {
                let mut hover_color = color;
                hover_color.r = (hover_color.r * 1.2).min(1.0);
                hover_color.g = (hover_color.g * 1.2).min(1.0);
                hover_color.b = (hover_color.b * 1.2).min(1.0);
                (hover_color, iced::Color::WHITE)
            }
            iced_widget::button::Status::Pressed => {
                let mut pressed_color = color;
                pressed_color.r *= 0.8;
                pressed_color.g *= 0.8;
                pressed_color.b *= 0.8;
                (pressed_color, iced::Color::from_rgb(0.8, 0.8, 0.8))
            }
            _ => (color, iced::Color::from_rgb(0.6, 0.6, 0.6))
        };

        iced_widget::button::Style {
            background: Some(iced::Background::Color(bg_color)),
            text_color: iced::Color::WHITE,
            border: iced::Border {
                color: border_color,
                width: 2.0,
                radius: iced::border::Radius::from(8.0),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        }
    });

    if let Some(message) = msg {
        btn = btn.on_press(message);
    }

    btn
}

pub fn render_action_bar(s: &PublicRoom, _your_seat: Option<usize>, your_turn: bool)
                         -> iced::Element<'static, Msg> {
    let mut bar = column![text("Actions").size(18)].spacing(8);

    if s.phase == Phase::Lobby {
        return bar.push(text("Waiting in lobby…").size(14)).into();
    }
    if !your_turn {
        return bar.push(text("Waiting for other players…").size(14)).into();
    }

    if s.in_betting {
        if s.current_bet == 0 {
            bar = bar.push(
                row![
                    poker_button("Check".to_string(), iced::Color::from_rgb(0.2, 0.6, 0.8), Some(Msg::Check)), // Blue for check
                    Space::with_width(Length::Fixed(12.0)),
                    poker_button(
                        format!("Bet ${}", if s.round <= 2 { s.limit_small } else { s.limit_big }),
                        iced::Color::from_rgb(0.8, 0.4, 0.2), // Orange for bet
                        Some(Msg::Bet)
                    ),
                ].spacing(8),
            );
        } else {
            let can_raise = s.raises_made < s.max_raises;
            bar = bar.push(
                column![
                    row![
                        poker_button("Call".to_string(), iced::Color::from_rgb(0.2, 0.7, 0.3), Some(Msg::Call)), // Green for call
                        Space::with_width(Length::Fixed(12.0)),
                        poker_button(
                            format!("Raise +${}", if s.round <= 2 { s.limit_small } else { s.limit_big }),
                            if can_raise { iced::Color::from_rgb(0.8, 0.4, 0.2) } else { iced::Color::from_rgb(0.5, 0.5, 0.5) },
                            can_raise.then_some(Msg::Raise)
                        ),
                    ].spacing(8),
                    Space::with_height(Length::Fixed(8.0)),
                    row![
                        poker_button("Fold".to_string(), iced::Color::from_rgb(0.8, 0.2, 0.2), Some(Msg::Fold)), // Red for fold
                    ],
                ].spacing(4),
            );
        }
    } else {
        bar = bar.push(
            column![
                row![
                    poker_button("Hit".to_string(), iced::Color::from_rgb(0.2, 0.6, 0.8), Some(Msg::TakeCard)), // Blue for hit
                    Space::with_width(Length::Fixed(12.0)),
                    poker_button("Stand".to_string(), iced::Color::from_rgb(0.6, 0.4, 0.8), Some(Msg::Stand)), // Purple for stand
                ].spacing(8),
                Space::with_height(Length::Fixed(8.0)),
                row![
                    poker_button("Fold".to_string(), iced::Color::from_rgb(0.8, 0.2, 0.2), Some(Msg::Fold)), // Red for fold
                ],
            ].spacing(4),
        );
    }

    bar.into()
}