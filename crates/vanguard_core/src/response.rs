use hyper::{Response, StatusCode};
use http_body_util::Full;
use bytes::Bytes;
use maud::Markup;

pub trait IntoResponse {
    fn into_response(self) -> Response<Full<Bytes>>;
}

impl IntoResponse for () {
    fn into_response(self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::OK)
            .body(Full::new(Bytes::new()))
            .unwrap()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from(self)))
            .unwrap()
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from(self)))
            .unwrap()
    }
}

impl IntoResponse for Markup {
    fn into_response(self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", "text/html")
            .body(Full::new(Bytes::from(self.into_string())))
            .unwrap()
    }
}

impl IntoResponse for Response<Full<Bytes>> {
    fn into_response(self) -> Response<Full<Bytes>> {
        self
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response<Full<Bytes>> {
        match self {
            Ok(v) => v.into_response(),
            Err(e) => e.into_response(),
        }
    }
}

impl IntoResponse for hyper::Error {
    fn into_response(self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from(self.to_string())))
            .unwrap()
    }
}

// Common App Error Tuple: (StatusCode, String)
impl IntoResponse for (StatusCode, String) {
    fn into_response(self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(self.0)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from(self.1)))
            .unwrap()
    }
}

impl IntoResponse for (StatusCode, &'static str) {
    fn into_response(self) -> Response<Full<Bytes>> {
        Response::builder()
            .status(self.0)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from(self.1)))
            .unwrap()
    }
}
