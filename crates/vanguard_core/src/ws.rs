use hyper::{Request, Response, StatusCode};
use hyper::body::Incoming;
use http_body_util::Full;
use bytes::Bytes;
use hyper_tungstenite::{is_upgrade_request, upgrade, HyperWebsocket};
pub use tokio_tungstenite::tungstenite::Message as WsMessage;

pub struct WsUpgrade {
    pub response: Response<Full<Bytes>>,
    pub websocket: HyperWebsocket,
}

pub fn is_websocket_upgrade(req: &Request<Incoming>) -> bool {
    is_upgrade_request(req)
}

pub fn upgrade_connection(mut req: Request<Incoming>) -> Result<WsUpgrade, Response<Full<Bytes>>> {
    if !is_upgrade_request(&req) {
        return Err(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from("Expected WebSocket upgrade request")))
            .unwrap());
    }

    let (response, websocket) = match upgrade(&mut req, None) {
        Ok(t) => t,
        Err(_) => {
            return Err(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Full::new(Bytes::from("WebSocket upgrade error")))
                .unwrap());
        }
    };

    let (parts, _body) = response.into_parts();
    let res = Response::from_parts(parts, Full::new(Bytes::new()));

    Ok(WsUpgrade {
        response: res,
        websocket,
    })
}
