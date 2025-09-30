use iced::futures::{channel::mpsc, SinkExt, StreamExt};
use iced::Subscription;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use crate::messages::Msg;
use cctmog_protocol::{ClientToServer, ServerToClient};

pub fn subscription(url: String, room: String, name: String) -> Subscription<Msg> {
    let id = format!("ws:{url}:{room}:{name}");
    let stream = iced::stream::channel(100, move |mut output| async move {
        match connect_async(url.clone()).await {
            Ok((mut ws, _)) => {
                let (tx_out, mut rx_out) = mpsc::unbounded::<ClientToServer>();
                let _ = output.send(Msg::WsConnected(tx_out.clone())).await;

                let join = ClientToServer::Join { room: room.clone(), name: name.clone() };
                let _ = ws.send(Message::Text(serde_json::to_string(&join).unwrap())).await;

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
            Err(e) => { let _ = output.send(Msg::WsError(format!("connect: {e:?}"))).await; }
        }
    });
    iced::Subscription::run_with_id(id, stream)
}