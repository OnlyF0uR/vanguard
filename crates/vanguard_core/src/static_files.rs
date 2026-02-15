use std::path::PathBuf;
use hyper::{Response, StatusCode};
use http_body_util::Full;
use bytes::Bytes;
use tokio::fs;
use crate::router::{BoxFuture, HandlerResponse, Ctx};

pub fn static_handler<S: Send + Sync + 'static>(root: &str) -> impl Fn(Ctx<S>) -> BoxFuture<HandlerResponse> + Send + Sync {
    let root = PathBuf::from(root);
    move |ctx| {
        let root = root.clone();
        Box::pin(async move {
            let path = ctx.req.uri().path();
            if path.contains("..") {
                 return Ok(Response::builder().status(StatusCode::FORBIDDEN).body(Full::new(Bytes::from("Forbidden"))).unwrap());
            }
            let rel_path = path.trim_start_matches('/');
            let file_path = root.join(rel_path);
            
            if file_path.exists() && file_path.is_file() {
                 if let Ok(contents) = fs::read(&file_path).await {
                    let mime_type = mime_guess::from_path(&file_path).first_or_octet_stream();
                    return Ok(Response::builder()
                        .header("Content-Type", mime_type.as_ref())
                        .body(Full::new(Bytes::from(contents)))
                        .unwrap());
                }
            }
             Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("Not Found")))
                .unwrap())
        }) as BoxFuture<HandlerResponse>
    }
}