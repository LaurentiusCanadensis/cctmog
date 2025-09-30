mod app;
mod embedded_server;
mod game;
mod games;
mod messages;
mod states;
use iced::border::Radius;
use iced::futures::channel::mpsc; // unbounded
use iced::futures::{SinkExt, StreamExt};
use iced::widget::image::{self, Handle as Handle};
use iced::widget::{
    button, column, container, horizontal_rule, row, svg::Handle as svgHandle, text, text_input, Space, Svg,
};
use iced::{Alignment, Border, Color, Element, Length, Renderer, Subscription, Task, Theme};
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uuid::Uuid;

use cctmog_protocol::*;

// Filesystem helper for card assets
use std::path::Path as FsPath;

// Canvas (Iced 0.13)
use iced::mouse;
use iced::widget::canvas::{
    self, stroke, Canvas, Frame, Geometry, Path as CanvasPath, Stroke, Text as CanvasText,
};
use iced_widget::Image;
/* ============================ Messages & App ============================ */
use app::App;   // <-- bring App into scope

// client/src/main.rs
mod types;
mod ui;
mod ws;


use crate::messages::Msg;

fn main() -> iced::Result {
    iced::application("cctmog (client)", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| iced::Theme::Dark)
        .run()
}

/* ================================ Theme ================================ */

pub const FELT: Color = Color {
    r: 0.070,
    g: 0.345,
    b: 0.190,
    a: 1.0,
};
pub const FELT_DARK: Color = Color {
    r: 0.045,
    g: 0.220,
    b: 0.120,
    a: 1.0,
};
pub const LIP: Color = Color {
    r: 0.020,
    g: 0.090,
    b: 0.050,
    a: 1.0,
};
pub const GOLD: Color = Color {
    r: 0.980,
    g: 0.860,
    b: 0.220,
    a: 1.0,
};
pub const INK: Color = Color {
    r: 0.10,
    g: 0.10,
    b: 0.11,
    a: 1.0,
};
pub const INK_SOFT: Color = Color {
    r: 0.14,
    g: 0.14,
    b: 0.16,
    a: 1.0,
};
pub const TEXT: Color = Color {
    r: 0.92,
    g: 0.92,
    b: 0.94,
    a: 1.0,
};
pub const ACCENT_BLUE: Color = Color {
    r: 0.36,
    g: 0.62,
    b: 0.98,
    a: 1.0,
};

fn brand_logo() -> iced::Element<'static, crate::messages::Msg> {
    // Embed the PNG at compile time to avoid path issues.
    // Adjust the path to where the file actually lives in your repo.
    const BYTES: &[u8] = include_bytes!("../../client/assets/cctmog_logo.png");

    let logo: Element<_> = Image::new(Handle::from_path("client/assets/cctmog_logo.png"))
        .width(Length::Fixed(220.0))
        .into();
    logo
}
pub fn plate() -> container::Style {
    container::Style {
        background: Some(iced::Background::Color(INK_SOFT)),
        border: Border {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.50),
            width: 1.0,
            radius: Radius::from(10.0),
        },
        text_color: Some(TEXT),
        ..Default::default()
    }
}

/* ============================== Table UI ============================== */

const MAX_SEATS: usize = 8;

#[derive(Copy, Clone)]
enum CardSize {
    Small,
    Large,
}

fn card_px(size: CardSize) -> (f32, f32) {
    match size {
        CardSize::Small => (50.0, 70.0),
        CardSize::Large => (95.0, 133.0),
    }
}

fn card_filename(card: &Card) -> String {
    let r = match card.rank {
        Rank::Ace => "1",
        Rank::Two => "2",
        Rank::Three => "3",
        Rank::Four => "4",
        Rank::Five => "5",
        Rank::Six => "6",
        Rank::Seven => "7",
        Rank::Eight => "8",
        Rank::Nine => "9",
        Rank::Ten => "10",
        Rank::Jack => "11",
        Rank::Queen => "12",
        Rank::King => "13",
    };
    let s = match card.suit {
        Suit::Clubs => "c",
        Suit::Diamonds => "d",
        Suit::Hearts => "h",
        Suit::Spades => "s",
    };
    format!("{}{}.svg", r, s)
}

fn card_svg(card: &Card, size: CardSize) -> Element<'static, Msg> {
    let path = format!("client/cards/{}", card_filename(card));
    let (w, h) = card_px(size);

    if FsPath::new(&path).exists() {
        Svg::new(svgHandle::from_path(path))
            .width(Length::Fixed(w))
            .height(Length::Fixed(h))
            .into()
    } else {
        container(text("❓").size(24))
            .width(Length::Fixed(w))
            .height(Length::Fixed(h))
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

fn cards_row_svg(cards: &[Card], size: CardSize, gap: f32) -> Element<'static, Msg> {
    if cards.is_empty() {
        return text("—").size(18).into();
    }
    let mut r = row![];
    for c in cards {
        r = r.push(card_svg(c, size));
        r = r.push(Space::with_width(Length::Fixed(gap)));
    }
    r.into()
}

/* ======================== Canvas: Felt & Pot ========================= */

#[derive(Debug, Clone, Copy)]
pub struct PokerTableCanvas {
    pub pot: u64,
    pub seats: usize,
    pub to_act_seat: Option<usize>, // None in lobby
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
        let r = bounds.width.min(bounds.height) * 0.36;

        // felt
        let circle = CanvasPath::circle(iced::Point::new(cx, cy), r);
        frame.fill(&circle, FELT);

        // subtle inner ring
        let inner = CanvasPath::circle(iced::Point::new(cx, cy), r * 0.97);
        frame.stroke(
            &inner,
            Stroke {
                width: 8.0,
                ..Default::default()
            },
        );

        // dark lip
        frame.stroke(
            &circle,
            Stroke {
                width: 14.0,
                ..Default::default()
            },
        );
        let lip = CanvasPath::circle(iced::Point::new(cx, cy), r + 7.0);
        frame.stroke(
            &lip,
            Stroke {
                width: 2.0,
                ..Default::default()
            },
        );

        // center pot badge
        let chip_r = r * 0.085;
        let stack = 3usize.min(((self.pot / 20).max(1)) as usize);
        for i in 0..stack {
            let p = iced::Point::new(
                cx + (i as f32) * (chip_r * 0.25),
                cy - (i as f32) * (chip_r * 0.15),
            );
            let c = CanvasPath::circle(p, chip_r);
            frame.fill(&c, GOLD);
            frame.stroke(
                &c,
                Stroke {
                    width: 1.0,
                    ..Default::default()
                },
            );
        }
        frame.fill_text(canvas::Text {
            content: format!("Pot {}", self.pot),
            position: iced::Point::new(cx, cy + chip_r * 1.8),
            size: iced::Pixels(chip_r * 0.9),
            ..Default::default()
        });

        vec![frame.into_geometry()]
    }
}

/* ============================== Logo ============================== */
pub fn logo_widget() -> Element<'static, Msg> {
    Image::new(Handle::from_path("client/assets/cctmog_logo.png"))
        .width(Length::Fixed(220.0))
        .height(Length::Fixed(120.0))
        .into()
}


fn chip_stack_icon() -> iced::Element<'static, crate::messages::Msg> {
    // 3 little circles to suggest stacks
    use iced::widget::canvas::{self, Canvas, Frame, Path as CanvasPath, Stroke};
    struct ChipIcon;
    impl<Message> canvas::Program<Message> for ChipIcon {
        type State = ();
        fn draw(
            &self,
            _: &Self::State,
            renderer: &iced::Renderer,
            _: &iced::Theme,
            bounds: iced::Rectangle,
            _: iced::mouse::Cursor,
        ) -> Vec<canvas::Geometry> {
            let mut frame = Frame::new(renderer, bounds.size());
            let r = bounds.height.min(bounds.width) * 0.22;
            let cx = bounds.width * 0.5;
            let cy = bounds.height * 0.5;

            let red = iced::Color {
                r: 0.85,
                g: 0.20,
                b: 0.20,
                a: 1.0,
            };
            let blue = iced::Color {
                r: 0.20,
                g: 0.45,
                b: 0.85,
                a: 1.0,
            };
            let white = iced::Color {
                r: 0.95,
                g: 0.95,
                b: 0.95,
                a: 1.0,
            };

            for (dx, c) in [(-r * 0.9, red), (0.0, blue), (r * 0.9, white)].into_iter() {
                let p = iced::Point::new(cx + dx, cy);
                let path = CanvasPath::circle(p, r);
                frame.fill(&path, c);
                frame.stroke(
                    &path,
                    Stroke {
                        width: 1.0,
                        ..Default::default()
                    },
                );
            }
            vec![frame.into_geometry()]
        }
    }
    Canvas::new(ChipIcon)
        .width(Length::Fixed(26.0))
        .height(Length::Fixed(20.0))
        .into()
}

pub fn seat_plate(
    p: &cctmog_protocol::PublicPlayer,
    is_you: bool,
    is_to_act: bool,
) -> iced::Element<'static, crate::messages::Msg> {
    let name = if is_you {
        format!("{} (you)", p.name)
    } else {
        p.name.clone()
    };

    let header = row![
        chip_stack_icon(),
        Space::with_width(Length::Fixed(8.0)),
        text(name).size(16),
        Space::with_width(Length::Fill),
        text(format!("{}", p.chips)).size(16),
    ]
    .align_y(Alignment::Center)
    .spacing(8);

    let act_hint: Element<Msg> = if is_to_act {
        // Text::style requires a closure producing a Style; set only color.
        text("●")
            .style(|_| iced_widget::text::Style {
                color: Some(TEXT),
                ..Default::default()
            })
            .size(14)
            .into()
    } else {
        Space::with_width(Length::Fixed(0.0)).into()
    };

    let plate = container(row![header, act_hint].spacing(8))
        .padding([8.0, 12.0]) // 2-value padding is valid
        .style(|_| plate()); // use local plate() helper

    let up = row![
        text("Up:").size(14),
        Space::with_width(Length::Fixed(6.0)),
        crate::cards_row_svg(&p.up_cards, crate::CardSize::Small, 6.0)
    ]
    .align_y(Alignment::Center);

    column![plate, Space::with_height(Length::Fixed(6.0)), up]
        .spacing(4)
        .width(Length::Shrink)
        .into()
}

fn seat_panel(p: &PublicPlayer, is_you: bool, is_to_act: bool) -> Element<'static, Msg> {
    let title = if is_you {
        format!("Seat {} · {} (you)", p.seat, p.name)
    } else {
        format!("Seat {} · {}", p.seat, p.name)
    };
    let turn = if is_to_act { " ⏳" } else { "" };
    let ready = if p.ready { "READY" } else { "" };

    container(
        column![
            row![
                text(title).size(16),
                Space::with_width(Length::Fixed(6.0)),
                text(ready).size(14),
                text(turn).size(16),
                Space::with_width(Length::Fill),
                text(format!("chips: {}", p.chips)).size(14),
            ]
            .align_y(Alignment::Center),
            Space::with_height(Length::Fixed(4.0)),
            row![
                text("Up:").size(14),
                Space::with_width(Length::Fixed(6.0)),
                cards_row_svg(&p.up_cards, CardSize::Small, 6.0)
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(4),
    )
    .padding(6)
    .width(Length::Fill)
    .into()
}

// ======================== Round table layout =========================

pub fn round_table_view(
    s: &PublicRoom,
    your_id: Option<Uuid>,
    your_seat: Option<usize>,
    _your_hand: &cctmog_protocol::PrivateHand, // unused here
) -> Element<'static, Msg> {
    // collect other players; "you" isn't rendered here specially
    let mut others: Vec<&PublicPlayer> = vec![];
    for p in &s.players {
        if Some(p.seat) != your_seat && Some(p.id) != your_id {
            others.push(p);
        }
    }

    // slots
    let mut it = others.into_iter();
    let tl = it.next();
    let tc = it.next();
    let tr = it.next();
    let rl = it.next();
    let ll = it.next();

    let seat_box = |pp: Option<&PublicPlayer>| -> Element<Msg> {
        match pp {
            Some(p) => {
                let you = your_id == Some(p.id);
                let to_act = s.to_act_seat == p.seat;
                container(seat_panel(p, you, to_act))
                    .width(Length::Shrink)
                    .into()
            }
            None => Space::with_width(Length::Fixed(0.0)).into(),
        }
    };

    // top band
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

    // middle band (left / spacer / right) — leave space for big Canvas in `view()`
    let mid_band = row![
        container(seat_box(ll))
            .width(Length::FillPortion(1))
            .align_x(Alignment::Start),
        Space::with_width(Length::FillPortion(2)),
        container(seat_box(rl))
            .width(Length::FillPortion(1))
            .align_x(Alignment::End),
    ]
    .spacing(12)
    .height(Length::FillPortion(2))
    .width(Length::Fill);

    // optional lobby hint
    let lobby_note: Element<Msg> = if s.phase == cctmog_protocol::Phase::Lobby {
        container(
            iced::widget::text("Waiting for players to click “I’m ready”. Dealer starts the hand.")
                .size(16),
        )
        .padding(8)
        .center_x(Length::Fill)
        .into()
    } else {
        Space::with_height(Length::Fixed(0.0)).into()
    };

    column![
        lobby_note,
        top_band,
        Space::with_height(Length::Fixed(8.0)),
        mid_band,
    ]
    .spacing(6)
    .width(Length::Fill)
    .into()
}

/* ============================== Helpers =============================== */

fn pill(label: String) -> Element<'static, Msg> {
    container(text(label).size(16)).padding([6.0, 10.0]).into()
}

// client/src/ui/actions.rs (or wherever you keep it)
pub fn render_action_bar(
    s: &PublicRoom,
    _your_seat: Option<usize>,
    your_turn: bool,
) -> Element<'static, Msg> {
    use iced::widget::{button, column, row, text, Space};
    use iced::Length;

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
                    button(text("Check"))
                        .on_press(Msg::Check)
                        .padding([10_u16, 18_u16]),
                    Space::with_width(Length::Fixed(8.0)),
                    button(text(format!(
                        "Bet {}",
                        if s.round <= 2 {
                            s.limit_small
                        } else {
                            s.limit_big
                        }
                    )))
                    .on_press(Msg::Bet)
                    .padding([10_u16, 18_u16]),
                ]
                .spacing(8),
            );
        } else {
            let can_raise = s.raises_made < s.max_raises;
            bar = bar.push(
                row![
                    button(text("Call"))
                        .on_press(Msg::Call)
                        .padding([10_u16, 18_u16]),
                    Space::with_width(Length::Fixed(8.0)),
                    button(text(format!(
                        "Raise +{}",
                        if s.round <= 2 {
                            s.limit_small
                        } else {
                            s.limit_big
                        }
                    )))
                    .on_press_maybe(can_raise.then_some(Msg::Raise))
                    .padding([10_u16, 18_u16]),
                    Space::with_width(Length::Fixed(8.0)),
                    button(text("Fold"))
                        .on_press(Msg::Fold)
                        .padding([10_u16, 18_u16]),
                ]
                .spacing(8),
            );
        }
    } else {
        bar = bar.push(
            row![
                button(text("Take card"))
                    .on_press(Msg::TakeCard)
                    .padding([10_u16, 18_u16]),
                Space::with_width(Length::Fixed(8.0)),
                button(text("Stand"))
                    .on_press(Msg::Stand)
                    .padding([10_u16, 18_u16]),
                Space::with_width(Length::Fixed(8.0)),
                button(text("Fold"))
                    .on_press(Msg::Fold)
                    .padding([10_u16, 18_u16]),
            ]
            .spacing(8),
        );
    }

    bar.into()
}

/* ======================= WebSocket subscription ======================= */

fn asset_test_panel() -> Element<'static, Msg> {
    use Rank::*;
    use Suit::*;
    let samples = [
        Card {
            rank: Ace,
            suit: Spades,
            face_up: true,
        },
        Card {
            rank: Ten,
            suit: Hearts,
            face_up: true,
        },
        Card {
            rank: Jack,
            suit: Diamonds,
            face_up: true,
        },
        Card {
            rank: Queen,
            suit: Clubs,
            face_up: true,
        },
    ];

    let row_large = samples.iter().fold(row![], |r, c| {
        r.push(card_svg(c, CardSize::Large))
            .push(Space::with_width(Length::Fixed(10.0)))
    });

    let row_small = samples.iter().fold(row![], |r, c| {
        r.push(card_svg(c, CardSize::Small))
            .push(Space::with_width(Length::Fixed(6.0)))
    });

    container(
        column![
            text("Card Test (client-side)").size(16),
            horizontal_rule(1),
            row_large,
            Space::with_height(Length::Fixed(8.0)),
            row_small,
            Space::with_height(Length::Fixed(8.0)),
            text("If any SVG is missing you will see ❓.").size(14),
        ]
        .spacing(8),
    )
    .padding(8)
    .width(Length::Fill)
    .into()
}

fn websocket_subscription(url: String, room: String, name: String) -> Subscription<Msg> {
    let id = format!("ws:{url}:{room}:{name}");
    let stream = iced::stream::channel(100, move |mut output| async move {
        match connect_async(url.clone()).await {
            Ok((mut ws, _)) => {
                // UI → WS
                let (tx_out, mut rx_out) = mpsc::unbounded::<ClientToServer>();
                let _ = output.send(Msg::WsConnected(tx_out.clone())).await;

                // Join immediately
                let join = ClientToServer::Join {
                    room: room.clone(),
                    name: name.clone(),
                };
                let _ = ws
                    .send(Message::Text(serde_json::to_string(&join).unwrap()))
                    .await;

                loop {
                    tokio::select! {
                        Some(cmd) = rx_out.next() => {
                            let _ = ws.send(Message::Text(serde_json::to_string(&cmd).unwrap())).await;
                        }
                        Some(Ok(msg)) = ws.next() => {
                            if let Message::Text(t) = msg {
                                match serde_json::from_str::<ServerToClient>(&t) {
                                    Ok(ev) => { let _ = output.send(Msg::WsEvent(ev)).await; }
                                    Err(e) => { let _ = output.send(Msg::WsError(format!("decode: {e}"))).await; }
                                }
                            }
                        }
                        else => break,
                    }
                }
                let _ = output.send(Msg::WsError("socket closed".into())).await;
            }
            Err(e) => {
                let _ = output.send(Msg::WsError(format!("connect: {e:?}"))).await;
            }
        }
    });

    iced::Subscription::run_with_id(id, stream)
}

/* ======================== Canvas: Felt & Pot ========================= */
