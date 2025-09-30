use iced::mouse;
use iced::widget::canvas::{self, Canvas, Frame, Path as CanvasPath, Stroke};
use iced::{Element};
use crate::messages::Msg;
use super::theme::FELT;
use cctmog_protocol::Card;

#[derive(Debug, Clone)]
pub struct PokerTableCanvas {
    pub pot: u64,
    pub seats: usize,
    pub to_act_seat: Option<usize>, // None in lobby
    pub community_cards: Vec<Card>, // Community cards for display
}

impl<Message> canvas::Program<Message> for PokerTableCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let cx = bounds.width * 0.5;
        let cy = bounds.height * 0.46;
        let r  = bounds.width.min(bounds.height) * 0.36;

        let circle = CanvasPath::circle(iced::Point::new(cx, cy), r);
        frame.fill(&circle, FELT);

        let inner = CanvasPath::circle(iced::Point::new(cx, cy), r * 0.97);
        frame.stroke(&inner, Stroke { width: 8.0, ..Default::default() });

        frame.stroke(&circle, Stroke { width: 14.0, ..Default::default() });
        let lip = CanvasPath::circle(iced::Point::new(cx, cy), r + 7.0);
        frame.stroke(&lip, Stroke { width: 2.0, ..Default::default() });

        // simple chip stack
        let chip_r = r * 0.085;
        let gold   = iced::Color { r: 0.980, g: 0.860, b: 0.220, a: 1.0 };
        let stack  = 3usize.min(((self.pot / 20).max(1)) as usize);
        for i in 0..stack {
            let p = iced::Point::new(cx + (i as f32) * (chip_r * 0.25), cy - (i as f32) * (chip_r * 0.15));
            let c = CanvasPath::circle(p, chip_r);
            frame.fill(&c, gold);
            frame.stroke(&c, Stroke { width: 1.0, ..Default::default() });
        }

        // Draw community cards if any
        if !self.community_cards.is_empty() {
            let card_width = r * 0.2;
            let card_height = card_width * 1.4;
            let card_spacing = card_width * 0.1;
            let total_width = (card_width * self.community_cards.len() as f32) + (card_spacing * (self.community_cards.len() - 1) as f32);
            let start_x = cx - (total_width / 2.0);

            for (i, card) in self.community_cards.iter().enumerate() {
                let card_x = start_x + (i as f32) * (card_width + card_spacing);
                let card_y = cy - (card_height / 2.0);

                // Draw card background
                let card_path = CanvasPath::rectangle(
                    iced::Point::new(card_x, card_y),
                    iced::Size::new(card_width, card_height)
                );
                frame.fill(&card_path, iced::Color::WHITE);
                frame.stroke(&card_path, Stroke { width: 1.0, ..Default::default() });

                // Draw card text (simplified - rank and suit)
                let card_text = format!("{}{}",
                    match card.rank {
                        cctmog_protocol::Rank::Ace => "A",
                        cctmog_protocol::Rank::Two => "2",
                        cctmog_protocol::Rank::Three => "3",
                        cctmog_protocol::Rank::Four => "4",
                        cctmog_protocol::Rank::Five => "5",
                        cctmog_protocol::Rank::Six => "6",
                        cctmog_protocol::Rank::Seven => "7",
                        cctmog_protocol::Rank::Eight => "8",
                        cctmog_protocol::Rank::Nine => "9",
                        cctmog_protocol::Rank::Ten => "T",
                        cctmog_protocol::Rank::Jack => "J",
                        cctmog_protocol::Rank::Queen => "Q",
                        cctmog_protocol::Rank::King => "K",
                    },
                    match card.suit {
                        cctmog_protocol::Suit::Spades => "♠",
                        cctmog_protocol::Suit::Hearts => "♥",
                        cctmog_protocol::Suit::Diamonds => "♦",
                        cctmog_protocol::Suit::Clubs => "♣",
                    }
                );

                let text_color = match card.suit {
                    cctmog_protocol::Suit::Hearts | cctmog_protocol::Suit::Diamonds => iced::Color::from_rgb(0.8, 0.1, 0.1),
                    _ => iced::Color::BLACK,
                };

                frame.fill_text(canvas::Text {
                    content: card_text,
                    position: iced::Point::new(card_x + card_width/2.0, card_y + card_height/2.0),
                    size: iced::Pixels(card_width * 0.3),
                    horizontal_alignment: iced::alignment::Horizontal::Center,
                    vertical_alignment: iced::alignment::Vertical::Center,
                    color: text_color,
                    ..Default::default()
                });
            }
        }

        // Draw pot text below community cards or centered if no cards
        let pot_y = if self.community_cards.is_empty() {
            cy + chip_r * 1.8
        } else {
            cy + r * 0.3
        };

        frame.fill_text(canvas::Text {
            content: format!("Pot: ${}", self.pot),
            position: iced::Point::new(cx, pot_y),
            size: iced::Pixels(chip_r * 0.8),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            vertical_alignment: iced::alignment::Vertical::Center,
            ..Default::default()
        });

        vec![frame.into_geometry()]
    }
}

pub fn felt(pot: u64, seats: usize, to_act_seat: Option<usize>) -> Element<'static, Msg> {
    Canvas::new(PokerTableCanvas { pot, seats, to_act_seat, community_cards: vec![] })
        .width(iced::Length::Fill)
        .height(iced::Length::Fixed(380.0))
        .into()
}

/// Enhanced felt with community cards support
pub fn felt_with_community(pot: u64, seats: usize, to_act_seat: Option<usize>, community_cards: Vec<Card>) -> Element<'static, Msg> {
    Canvas::new(PokerTableCanvas { pot, seats, to_act_seat, community_cards })
        .width(iced::Length::Fill)
        .height(iced::Length::Fixed(380.0))
        .into()
}