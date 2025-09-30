use iced::{Alignment, Element, Length};
use iced_widget::{container, row, text, Space};

use cctmog_protocol::{PublicRoom, Phase};
use crate::messages::Msg;

pub fn header_view(room: &PublicRoom) -> Element<'static, Msg> {
    let room_info = text(format!("Room: {} | Phase: {}",
        room.room.chars().take(8).collect::<String>(),
        match room.phase {
            Phase::Lobby => "Lobby",
            Phase::Acting => "Acting",
            Phase::Dealing => "Dealing",
            Phase::Showdown => "Showdown",
            Phase::Comments => "Comments",
            Phase::WaitingForDealer => "Waiting for Dealer",
            Phase::DealerSelection => "Dealer Selection",
            Phase::GameSelection => "Game Selection",
        }
    ))
    .size(14)
    .style(|_theme| iced_widget::text::Style {
        color: Some(iced::Color::from_rgb(0.9, 0.9, 0.9)),
        ..Default::default()
    });

    let pot_info = text(format!("Pot: ${}", room.pot))
        .size(14)
        .style(|_theme| iced_widget::text::Style {
            color: Some(iced::Color::from_rgb(0.2, 0.8, 0.2)),
            ..Default::default()
        });

    let player_count = text(format!("Players: {}/8", room.players.len()))
        .size(14)
        .style(|_theme| iced_widget::text::Style {
            color: Some(iced::Color::from_rgb(0.8, 0.8, 0.2)),
            ..Default::default()
        });

    container(
        row![
            room_info,
            Space::with_width(Length::Fill),
            pot_info,
            Space::with_width(Length::Fixed(20.0)),
            player_count,
        ]
        .align_y(Alignment::Center)
        .spacing(10)
    )
    .width(Length::Fixed(60.0 * 10.0)) // 60 units wide
    .height(Length::Fixed(10.0 * 10.0)) // 10 units high
    .padding([8, 12])
    .style(|_theme| iced_widget::container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.1, 0.1, 0.1, 0.9))),
        border: iced::Border {
            color: iced::Color::from_rgb(0.3, 0.3, 0.3),
            width: 1.0,
            radius: iced::border::Radius::from(6.0),
        },
        ..Default::default()
    })
    .into()
}