use hyper::{Response, StatusCode};
use http_body_util::Full;
use bytes::Bytes;
use vanguard_core::router::{HandlerResponse, Ctx};
use vanguard_core::ws::{is_websocket_upgrade, upgrade_connection, WsMessage};
use crate::backend::AppState;
use vanguard_core::futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
struct ClientCommand {
    action: String,
}

#[derive(Serialize)]
struct ServerUpdate {
    topic: String,
    data: String,
}

pub async fn ws_handler(ctx: Ctx<AppState>) -> HandlerResponse {
    // We should authenticate before upgrading
    let user = match super::handlers::get_user(&ctx) {
        Some(u) => u,
        None => {
            return Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(Full::new(Bytes::from("Unauthorized for WebSocket")))
                .unwrap());
        }
    };
    
    let req = ctx.req;

    if !is_websocket_upgrade(&req) {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from("Not a WebSocket connection request")))
            .unwrap());
    }

    let upgrade_res = match upgrade_connection(req) {
        Ok(upg) => upg,
        Err(err_resp) => return Ok(err_resp),
    };

    let state = ctx.state.clone();
    let user_sub = user.sub.clone();

    // Spawn a Tokio task so the router can immediately return the 101 Response
    tokio::spawn(async move {
        // Await the upgrade completion
        let mut websocket = match upgrade_res.websocket.await {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("WebSocket upgrade error: {}", e);
                return;
            }
        };

        // Send a welcome message
        let welcome = serde_json::to_string(&ServerUpdate {
            topic: "system".into(),
            data: format!("Welcome, {}", user_sub),
        }).unwrap();
        
        let _ = websocket.send(WsMessage::Text(welcome.into())).await;

        // Message loop
        while let Some(msg) = websocket.next().await {
            match msg {
                Ok(WsMessage::Text(text)) => {
                    if let Ok(cmd) = serde_json::from_str::<ClientCommand>(&text) {
                        if cmd.action == "ping" {
                            let resp = serde_json::to_string(&ServerUpdate {
                                topic: "pong".into(),
                                data: "PONG".into(),
                            }).unwrap();
                            let _ = websocket.send(WsMessage::Text(resp.into())).await;
                        } else if cmd.action == "increment_counter" {
                            let mut count = state.counters.entry(user_sub.clone()).or_insert(0);
                            *count += 1;
                            let new_val = *count;
                            
                            let update = serde_json::to_string(&ServerUpdate {
                                topic: "counter_update".into(),
                                data: new_val.to_string(),
                            }).unwrap();
                            
                            let _ = websocket.send(WsMessage::Text(update.into())).await;
                        }
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    break;
                }
                Err(e) => {
                    eprintln!("Websocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(upgrade_res.response)
}
