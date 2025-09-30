use iced::{Alignment, Element, Length};
use iced_widget::{column, container, row, text, Space};

use uuid::Uuid;
use cctmog_protocol::{PublicPlayer, PublicRoom};

use crate::messages::Msg;
use crate::ui::cards::{face_down_cards_row, cards_row_svg, CardSize};

fn player_avatar(name: &str, is_to_act: bool) -> Element<'static, Msg> {
    let avatar_color = match name.chars().next().unwrap_or('A') {
        'J' => iced::Color::from_rgb(0.2, 0.8, 0.4), // Green for John
        'F' => iced::Color::from_rgb(0.8, 0.3, 0.2), // Red for Frank
        'S' => iced::Color::from_rgb(0.3, 0.5, 0.9), // Blue for Santo
        'M' => iced::Color::from_rgb(0.9, 0.7, 0.2), // Yellow for Mass
        'D' => iced::Color::from_rgb(0.8, 0.2, 0.8), // Purple for Dan
        'L' => iced::Color::from_rgb(0.2, 0.8, 0.8), // Cyan for Lor
        _ => iced::Color::from_rgb(0.6, 0.6, 0.6), // Default gray
    };

    let border_color = if is_to_act {
        iced::Color::from_rgb(1.0, 0.8, 0.0) // Gold border for active player
    } else {
        iced::Color::from_rgb(0.4, 0.4, 0.4) // Gray border
    };

    container(
        container(
            text(name.chars().next().unwrap_or('?').to_uppercase().to_string())
                .size(14)
                .style(move |_theme| iced_widget::text::Style {
                    color: Some(iced::Color::WHITE),
                    ..Default::default()
                })
        )
        .width(Length::Fixed(32.0))
        .height(Length::Fixed(32.0))
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .style(move |_theme| iced_widget::container::Style {
            background: Some(iced::Background::Color(avatar_color)),
            border: iced::Border {
                color: border_color,
                width: if is_to_act { 2.0 } else { 1.0 },
                radius: iced::border::Radius::from(16.0), // Circle
            },
            ..Default::default()
        })
    )
    .width(Length::Fixed(36.0))
    .height(Length::Fixed(36.0))
    .center_x(Length::Fill)
    .into()
}

fn chip_stack(chips: u64) -> Element<'static, Msg> {
    if chips == 0 {
        return Space::with_width(Length::Fixed(0.0)).into();
    }

    let stack_height = (chips / 100).min(5).max(1) as usize;
    let chip_colors = [
        iced::Color::from_rgb(0.2, 0.8, 0.2), // Green for 100s
        iced::Color::from_rgb(0.2, 0.4, 0.8), // Blue for 500s
        iced::Color::from_rgb(0.8, 0.2, 0.2), // Red for 1000s
        iced::Color::from_rgb(0.8, 0.8, 0.2), // Yellow for 5000s
        iced::Color::from_rgb(0.6, 0.2, 0.8), // Purple for 10000s
    ];

    let mut chip_elements = vec![];
    for i in 0..stack_height {
        let color = chip_colors[i % chip_colors.len()];
        chip_elements.push(
            container(Space::with_width(Length::Fixed(0.0)))
                .width(Length::Fixed(20.0))
                .height(Length::Fixed(4.0))
                .style(move |_theme| iced_widget::container::Style {
                    background: Some(iced::Background::Color(color)),
                    border: iced::Border {
                        color: iced::Color::from_rgb(0.3, 0.3, 0.3),
                        width: 0.5,
                        radius: iced::border::Radius::from(2.0),
                    },
                    ..Default::default()
                })
                .into()
        );
    }

    container(
        column(chip_elements)
            .spacing(1)
            .align_x(Alignment::Center)
    )
    .width(Length::Fixed(24.0))
    .center_x(Length::Fill)
    .into()
}

fn seat_panel(p: &PublicPlayer, is_you: bool, is_to_act: bool) -> Element<'static, Msg> {
    let player_name = if is_you {
        format!("{} (You)", p.name)
    } else {
        p.name.clone()
    };

    let mut name_style = text(player_name).size(12);
    if is_to_act {
        name_style = text(format!("● {}", if is_you { format!("{} (You)", p.name) } else { p.name.clone() }))
            .size(12)
            .style(|_theme| iced_widget::text::Style {
                color: Some(iced::Color::from_rgb(1.0, 0.8, 0.0)), // Gold for active player
                ..Default::default()
            });
    }

    let chip_count = text(format!("${}", p.chips))
        .size(11)
        .style(|_theme| iced_widget::text::Style {
            color: Some(iced::Color::from_rgb(0.8, 0.8, 0.8)),
            ..Default::default()
        });

    let card_count_indicator: Option<Element<'static, Msg>> = if !is_you && p.cards_count > 0 {
        Some(text(format!("{} card{}", p.cards_count, if p.cards_count == 1 { "" } else { "s" }))
            .size(9)
            .style(|_theme: &iced::Theme| iced_widget::text::Style {
                color: Some(iced::Color::from_rgb(0.6, 0.9, 0.6)),
                ..Default::default()
            }).into())
    } else {
        None
    };

    let cards_row = if !is_you {
        let hole_card_count = if p.cards_count >= p.up_cards.len() {
            p.cards_count - p.up_cards.len()
        } else {
            0
        };

        let mut cards = vec![];

        if hole_card_count > 0 {
            cards.push(face_down_cards_row(hole_card_count, CardSize::Medium, 4.0));
        }

        if !p.up_cards.is_empty() {
            if hole_card_count > 0 {
                cards.push(Space::with_width(Length::Fixed(6.0)).into());
            }
            cards.push(cards_row_svg(&p.up_cards, CardSize::Medium, 4.0));
        }

        if !cards.is_empty() {
            row(cards).align_y(Alignment::Center).into()
        } else {
            text("—").size(12).style(|_theme| iced_widget::text::Style {
                color: Some(iced::Color::from_rgb(0.5, 0.5, 0.5)),
                ..Default::default()
            }).into()
        }
    } else if is_you && !p.up_cards.is_empty() {
        cards_row_svg(&p.up_cards, CardSize::Small, 2.0)
    } else {
        Space::with_height(Length::Fixed(0.0)).into()
    };

    container(
        column![
            row![
                player_avatar(&p.name, is_to_act),
                Space::with_width(Length::Fixed(8.0)),
                column![
                    name_style,
                    chip_count,
                    if let Some(indicator) = card_count_indicator {
                        indicator
                    } else {
                        Space::with_height(Length::Fixed(0.0)).into()
                    }
                ].spacing(2),
                Space::with_width(Length::Fill),
                chip_stack(p.chips),
            ]
            .align_y(Alignment::Center),
            Space::with_height(Length::Fixed(4.0)),
            cards_row,
        ]
        .spacing(2)
    )
    .padding([4, 8])
    .style(move |_theme| iced_widget::container::Style {
        background: if is_to_act {
            Some(iced::Background::Color(iced::Color::from_rgba(1.0, 0.8, 0.0, 0.1)))
        } else {
            Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.3)))
        },
        border: iced::Border {
            color: if is_to_act {
                iced::Color::from_rgb(1.0, 0.8, 0.0)
            } else {
                iced::Color::from_rgb(0.3, 0.3, 0.3)
            },
            width: if is_to_act { 2.0 } else { 1.0 },
            radius: iced::border::Radius::from(6.0),
        },
        ..Default::default()
    })
    .width(Length::Fixed(120.0))
    .into()
}

pub fn table_view(
    s: &PublicRoom,
    your_id: Option<Uuid>,
    your_seat: Option<usize>,
) -> Element<'static, Msg> {
    let mut others: Vec<&PublicPlayer> = vec![];
    for p in &s.players {
        let is_you = your_id.map(|id| id == p.id).unwrap_or(false)
            || your_seat.map(|seat| seat == p.seat).unwrap_or(false);
        if !is_you {
            others.push(p);
        }
    }

    let mut it = others.into_iter();
    let tl = it.next();
    let tc = it.next();
    let tr = it.next();
    let bl = it.next();
    let br = it.next();
    let ll = it.next();
    let rr = it.next();

    let seat_box = |pp: Option<&PublicPlayer>| -> Element<Msg> {
        match pp {
            Some(p) => {
                let you = your_id == Some(p.id) || your_seat == Some(p.seat);
                let to_act = s.to_act_seat == p.seat;
                seat_panel(p, you, to_act)
            }
            None => Space::with_width(Length::Fixed(0.0)).into(),
        }
    };

    let top_band = row![
        container(seat_box(tl)).width(Length::FillPortion(1)),
        container(seat_box(tc))
            .width(Length::FillPortion(1))
            .center_x(Length::Fill),
        container(seat_box(tr))
            .width(Length::FillPortion(1))
            .align_x(Alignment::End),
    ]
        .spacing(12)
        .width(Length::Fill);

    let felt_canvas = crate::ui::canvas::felt_with_community(
        s.pot,
        s.players.len(),
        if s.phase == cctmog_protocol::Phase::Lobby { None } else { Some(s.to_act_seat) },
        s.community_cards.clone(),
    );

    let mid_band = row![
        container(seat_box(ll))
            .width(Length::FillPortion(1))
            .align_x(Alignment::Start),
        container(felt_canvas)
            .width(Length::FillPortion(2))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
        container(seat_box(rr))
            .width(Length::FillPortion(1))
            .align_x(Alignment::End),
    ]
        .spacing(12)
        .height(Length::FillPortion(2))
        .width(Length::Fill);

    let bottom_band = row![
        container(seat_box(bl))
            .width(Length::FillPortion(1))
            .align_x(Alignment::Start),
        Space::with_width(Length::FillPortion(1)),
        container(seat_box(br))
            .width(Length::FillPortion(1))
            .align_x(Alignment::End),
    ]
        .spacing(12)
        .width(Length::Fill);

    container(
        column![
            top_band,
            Space::with_height(Length::Fixed(6.0)),
            mid_band,
            Space::with_height(Length::Fixed(6.0)),
            bottom_band,
        ]
        .spacing(4)
        .width(Length::Fill)
    )
    .width(Length::Fixed(60.0 * 10.0)) // 60 units wide
    .height(Length::Fixed(40.0 * 10.0)) // 40 units high
    .style(|_theme| iced_widget::container::Style {
        background: Some(iced::Background::Color(iced::Color::from_rgba(0.0, 0.3, 0.0, 0.8))),
        border: iced::Border {
            color: iced::Color::from_rgb(0.3, 0.3, 0.3),
            width: 1.0,
            radius: iced::border::Radius::from(0.0),
        },
        ..Default::default()
    })
    .into()
}