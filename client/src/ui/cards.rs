use iced::{Element, Length};
use iced_widget::{row, container, text, Space, Svg};
use iced_widget::svg::Handle;
use std::path::Path as FsPath;
use cctmog_protocol::{Card, Rank, Suit};
use crate::messages::Msg;

#[derive(Copy, Clone)]
pub enum CardSize { Small, Medium, Large }

fn card_px(size: CardSize) -> (f32, f32) {
    match size {
        CardSize::Small => (50.0, 70.0),
        CardSize::Medium => (65.0, 91.0),
        CardSize::Large => (95.0, 133.0),
    }
}

fn filename(card: &Card) -> String {
    let r = match card.rank {
        Rank::Ace => "1", Rank::Two => "2", Rank::Three => "3", Rank::Four => "4",
        Rank::Five => "5", Rank::Six => "6", Rank::Seven => "7", Rank::Eight => "8",
        Rank::Nine => "9", Rank::Ten => "10", Rank::Jack => "11", Rank::Queen => "12", Rank::King => "13",
    };
    let s = match card.suit { Suit::Clubs=>"c", Suit::Diamonds=>"d", Suit::Hearts=>"h", Suit::Spades=>"s" };
    format!("{}{}.svg", r, s)
}

pub fn card_svg(card: &Card, size: CardSize) -> Element<'static, Msg> {
    let path = format!("client/cards/{}", filename(card));
    let (w, h) = card_px(size);
    if FsPath::new(&path).exists() {
        Svg::new(Handle::from_path(path)).width(Length::Fixed(w)).height(Length::Fixed(h)).into()
    } else {
        container(text("❓").size(24)).width(Length::Fixed(w)).height(Length::Fixed(h)).into()
    }
}

// Display a face-down card back
pub fn card_back_svg(size: CardSize) -> Element<'static, Msg> {
    let (w, h) = card_px(size);
    // Use a simple colored rectangle to represent card back
    container(text("CCTMOG").size(match size { CardSize::Small => 24.0, CardSize::Medium => 32.0, CardSize::Large => 48.0 }))
        .width(Length::Fixed(w))
        .height(Length::Fixed(h))
        .style(|_theme: &iced::Theme| iced_widget::container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.4, 0.8))), // Blue card back
            border: iced::Border {
                radius: 4.0.into(),
                width: 1.0,
                color: iced::Color::from_rgb(0.1, 0.1, 0.3),
            },
            ..Default::default()
        })
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}

// Display multiple face-down cards in a row
pub fn face_down_cards_row(count: usize, size: CardSize, gap: f32) -> Element<'static, Msg> {
    if count == 0 { return text("—").size(18).into(); }
    let mut r = row![];
    for _ in 0..count {
        r = r.push(card_back_svg(size)).push(Space::with_width(Length::Fixed(gap)));
    }
    r.into()
}

pub fn cards_row_svg(cards: &[Card], size: CardSize, gap: f32) -> Element<'static, Msg> {
    if cards.is_empty() { return text("—").size(18).into(); }
    let mut r = row![];
    for c in cards {
        // Show face-down cards as card backs, face-up cards normally
        if c.face_up {
            r = r.push(card_svg(c, size)).push(Space::with_width(Length::Fixed(gap)));
        } else {
            r = r.push(card_back_svg(size)).push(Space::with_width(Length::Fixed(gap)));
        }
    }
    r.into()
}