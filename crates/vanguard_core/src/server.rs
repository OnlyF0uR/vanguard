use std::net::SocketAddr;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;
use crate::router::Router;
use hyper::{Response, StatusCode, header};
use http_body_util::Full;
use bytes::Bytes;

pub struct ServerConfig {
    pub addr: SocketAddr,
    pub cors_domain: Option<String>,
}

pub async fn run<S: Send + Sync + 'static>(
    config: ServerConfig,
    router: Router<S>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(config.addr).await?;
    println!("Listening on http://{}", config.addr);

    let cors_domain = config.cors_domain.clone();

    loop {
        let (stream, _) = listener.accept().await?;
        let router = router.clone();
        let cors_domain = cors_domain.clone();

        tokio::task::spawn(async move {
            let peer_ip = match stream.peer_addr() {
                Ok(addr) => addr.ip(),
                Err(_) => "0.0.0.0".parse().unwrap(),
            };
            let io = TokioIo::new(stream);
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, hyper::service::service_fn(move |mut req| {
                    let router = router.clone();
                    let cors_domain = cors_domain.clone();
                    async move {
                        let origin = req.headers()
                            .get(header::ORIGIN)
                            .and_then(|v| v.to_str().ok())
                            .map(|s| s.to_string());
                            
                        let is_options = req.method() == hyper::Method::OPTIONS;
                        
                        let res = if is_options {
                            let mut res = Response::builder()
                                .status(StatusCode::NO_CONTENT)
                                .body(Full::new(Bytes::new()))
                                .unwrap();
                            apply_cors_headers(res.headers_mut(), origin.as_deref(), &cors_domain);
                            res
                        } else {
                            req.extensions_mut().insert(peer_ip);
                            let mut res = router.handle(req).await?;
                            apply_cors_headers(res.headers_mut(), origin.as_deref(), &cors_domain);
                            res
                        };
                        Ok::<_, hyper::Error>(res)
                    }
                }))
                .with_upgrades()
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn apply_cors_headers(headers: &mut header::HeaderMap, origin: Option<&str>, allowed_domain: &Option<String>) {
    let allow_origin = match (origin, allowed_domain) {
        (Some(org), _) if org.starts_with("http://localhost") || org.starts_with("http://127.0.0.1") => org,
        (Some(org), Some(allowed)) if org == allowed => org,
        (_, _) => "null",
    };

    if allow_origin != "null" {
        headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, header::HeaderValue::from_str(allow_origin).unwrap());
        headers.insert(header::ACCESS_CONTROL_ALLOW_METHODS, header::HeaderValue::from_static("GET, POST, OPTIONS"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_HEADERS, header::HeaderValue::from_static("Content-Type, Authorization"));
        headers.insert(header::ACCESS_CONTROL_ALLOW_CREDENTIALS, header::HeaderValue::from_static("true"));
    }
}