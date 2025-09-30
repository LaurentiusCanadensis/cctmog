// client/src/ui/table.rs
use iced::{Element, Length};
use iced_widget::column;

use uuid::Uuid;
use cctmog_protocol::PublicRoom;

use crate::messages::Msg;
use crate::ui::header::header_view;
use crate::ui::game_table::table_view;
use crate::ui::player_options::player_options_view;

/// Main table UI view combining header, table, and player options
pub fn full_table_view(
    room: &PublicRoom,
    your_id: Option<Uuid>,
    your_seat: Option<usize>,
    your_hand: &cctmog_protocol::PrivateHand,
) -> Element<'static, Msg> {
    column![
        header_view(room),
        table_view(room, your_id, your_seat),
        player_options_view(room, your_id, your_seat, your_hand),
    ]
    .spacing(0)
    .width(Length::Fixed(60.0 * 10.0)) // Total 60 units wide
    .height(Length::Fixed(70.0 * 10.0)) // Total 70 units high (10+40+20)
    .into()
}

/// Legacy table view for backwards compatibility
pub fn round_table_view(
    s: &PublicRoom,
    your_id: Option<Uuid>,
    your_seat: Option<usize>,
    your_hand: &cctmog_protocol::PrivateHand,
) -> Element<'static, Msg> {
    full_table_view(s, your_id, your_seat, your_hand)
}