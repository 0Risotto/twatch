//! WebSocket live-state transport.

use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::time::interval;

use twatch_core::gateway::{
    DownloadSnapshot, GatewayEvent, PlayerSnapshot, SessionSnapshot, ToastLevel,
};

use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum WsFrame {
    Snapshot {
        revision: u64,
        session: Option<SessionSnapshot>,
        downloads: Vec<DownloadSnapshot>,
        player: PlayerSnapshot,
    },
    Session {
        revision: u64,
        session: Option<SessionSnapshot>,
    },
    Downloads {
        revision: u64,
        downloads: Vec<DownloadSnapshot>,
    },
    Player {
        revision: u64,
        player: PlayerSnapshot,
    },
    Toast {
        revision: u64,
        level: ToastLevel,
        message: String,
    },
}

fn text_frame(frame: &WsFrame) -> Option<Message> {
    serde_json::to_string(frame).ok().map(|json| Message::Text(json.into()))
}

fn snapshot_frame(state: &AppState) -> WsFrame {
    WsFrame::Snapshot {
        revision: state.gateway.revision(),
        session: state.gateway.session(),
        downloads: state.gateway.downloads(),
        player: state.gateway.player(),
    }
}

/// `/api/ws` upgrade handler.
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut events = state.gateway.subscribe();

    if let Some(frame) = text_frame(&snapshot_frame(&state))
        && sender.send(frame).await.is_err()
    {
        return;
    }

    let mut heartbeat = interval(Duration::from_secs(20));
    heartbeat.tick().await;

    loop {
        tokio::select! {
            event = events.recv() => {
                let frame = match event {
                    Ok(GatewayEvent::SessionChanged) => WsFrame::Session {
                        revision: state.gateway.revision(),
                        session: state.gateway.session(),
                    },
                    Ok(GatewayEvent::DownloadsChanged) => WsFrame::Downloads {
                        revision: state.gateway.revision(),
                        downloads: state.gateway.downloads(),
                    },
                    Ok(GatewayEvent::PlayerChanged) => WsFrame::Player {
                        revision: state.gateway.revision(),
                        player: state.gateway.player(),
                    },
                    Ok(GatewayEvent::Toast { level, message }) => WsFrame::Toast {
                        revision: state.gateway.revision(),
                        level,
                        message,
                    },
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                        // The client is behind.  A full snapshot is cheaper than
                        // replaying a burst of deltas, and the revision guard on
                        // the client keeps ordering safe.
                        snapshot_frame(&state)
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                };
                if let Some(message) = text_frame(&frame)
                    && sender.send(message).await.is_err()
                {
                    break;
                }
            }
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Ping(payload))) => {
                        if sender.send(Message::Pong(payload)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) | Some(Ok(Message::Text(_))) | Some(Ok(Message::Binary(_))) => {}
                    Some(Err(_)) => break,
                }
            }
            _ = heartbeat.tick() => {
                if sender.send(Message::Ping(Vec::new().into())).await.is_err() {
                    break;
                }
            }
        }
    }
}
