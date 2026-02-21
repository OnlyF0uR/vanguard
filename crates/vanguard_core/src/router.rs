use std::collections::HashMap;
use std::sync::Arc;
use hyper::{Request, Response, StatusCode};
use hyper::body::Incoming;
use http_body_util::Full;
use bytes::Bytes;
use std::future::Future;
use std::pin::Pin;

pub type HandlerResponse = Result<Response<Full<Bytes>>, hyper::Error>;
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

// Context passed to handlers
pub struct Ctx<S> {
    pub req: Request<Incoming>,
    pub state: Arc<S>,
}

pub enum ExtractError {
    JsonSync(String),
    JsonDeserialize(String),
    FormSync(String),
    FormDeserialize(String),
}

impl crate::response::IntoResponse for ExtractError {
    fn into_response(self) -> Response<Full<Bytes>> {
        let (status, msg) = match self {
            Self::JsonSync(e) => (StatusCode::BAD_REQUEST, format!("Failed to read body: {}", e)),
            Self::JsonDeserialize(e) => (StatusCode::BAD_REQUEST, format!("JSON deserialize error: {}", e)),
            Self::FormSync(e) => (StatusCode::BAD_REQUEST, format!("Failed to read body: {}", e)),
            Self::FormDeserialize(e) => (StatusCode::BAD_REQUEST, format!("Form deserialize error: {}", e)),
        };
        (status, msg).into_response()
    }
}

impl<S> Ctx<S> {
    pub async fn json<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, ExtractError> {
        use http_body_util::BodyExt;
        let bytes = self.req.body_mut().collect().await.map_err(|e| ExtractError::JsonSync(e.to_string()))?.to_bytes();
        serde_json::from_slice(&bytes).map_err(|e| ExtractError::JsonDeserialize(e.to_string()))
    }

    pub async fn form<T: serde::de::DeserializeOwned>(&mut self) -> Result<T, ExtractError> {
        use http_body_util::BodyExt;
        let bytes = self.req.body_mut().collect().await.map_err(|e| ExtractError::FormSync(e.to_string()))?.to_bytes();
        serde_urlencoded::from_bytes(&bytes).map_err(|e| ExtractError::FormDeserialize(e.to_string()))
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.req.headers().get(name).and_then(|v| v.to_str().ok())
    }

    pub fn cookie(&self, name: &str) -> Option<String> {
        let cookie_header = self.header("cookie")?;
        for cookie_str in cookie_header.split(';') {
            let cookie_str = cookie_str.trim();
            if cookie_str.starts_with(&format!("{}=", name)) {
                return Some(cookie_str[name.len() + 1..].to_string());
            }
        }
        None
    }

    pub fn ip(&self) -> std::net::IpAddr {
        self.req.extensions().get::<std::net::IpAddr>().copied().unwrap_or_else(|| "0.0.0.0".parse().unwrap())
    }
}

pub type Handler<S> = Arc<dyn Fn(Ctx<S>) -> BoxFuture<HandlerResponse> + Send + Sync>;

#[derive(Clone, Hash, Eq, PartialEq)]
struct RouteKey {
    path: String,
    method: hyper::Method,
}

struct RouterInner<S> {
    routes: HashMap<RouteKey, Handler<S>>,
    prefix_routes: Vec<(String, Handler<S>)>,
    default_handler: Handler<S>,
    state: Arc<S>,
}

pub struct Router<S> {
    inner: Arc<RouterInner<S>>,
}

impl<S> Clone for Router<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<S: Send + Sync + 'static> Router<S> {
    pub fn new(state: S) -> Self {
        let state = Arc::new(state);
        let inner = RouterInner {
            routes: HashMap::new(),
            prefix_routes: Vec::new(),
            default_handler: Arc::new(|_ctx| Box::pin(async {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Full::new(Bytes::from("404 Not Found")))
                    .unwrap())
            })),
            state,
        };
        Self { inner: Arc::new(inner) }
    }

    pub fn get<F, Fut, R>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(Ctx<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: crate::response::IntoResponse + 'static,
    {
        let inner = Arc::get_mut(&mut self.inner).expect("Router cannot be modified after cloning");
        inner.routes.insert(
            RouteKey { path: path.to_string(), method: hyper::Method::GET },
            Arc::new(move |ctx| {
                let fut = handler(ctx);
                Box::pin(async move {
                    Ok(fut.await.into_response())
                })
            }),
        );
        self
    }

    pub fn post<F, Fut, R>(mut self, path: &str, handler: F) -> Self
    where
        F: Fn(Ctx<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: crate::response::IntoResponse + 'static,
    {
        let inner = Arc::get_mut(&mut self.inner).expect("Router cannot be modified after cloning");
        inner.routes.insert(
            RouteKey { path: path.to_string(), method: hyper::Method::POST },
            Arc::new(move |ctx| {
                let fut = handler(ctx);
                Box::pin(async move {
                    Ok(fut.await.into_response())
                })
            }),
        );
        self
    }

    pub fn mount<F, Fut, R>(mut self, prefix: &str, handler: F) -> Self
    where
        F: Fn(Ctx<S>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: crate::response::IntoResponse + 'static,
    {
        let clean_prefix = if prefix.len() > 1 && prefix.ends_with('/') {
            &prefix[..prefix.len() - 1]
        } else {
            prefix
        };

        let inner = Arc::get_mut(&mut self.inner).expect("Router cannot be modified after cloning");
        inner.prefix_routes.push((
            clean_prefix.to_string(),
            Arc::new(move |ctx| {
                let fut = handler(ctx);
                Box::pin(async move {
                    Ok(fut.await.into_response())
                })
            }),
        ));
        inner.prefix_routes
            .sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        self
    }

    pub async fn handle(&self, req: Request<Incoming>) -> HandlerResponse {
        let path = req.uri().path().to_string();
        let method = req.method().clone();
        
        if let Some(handler) = self.inner.routes.get(&RouteKey { path: path.clone(), method }) {
            let ctx = Ctx { req, state: self.inner.state.clone() };
            return handler(ctx).await;
        }

        for (prefix, handler) in &self.inner.prefix_routes {
            if path.starts_with(prefix) {
                let ctx = Ctx { req, state: self.inner.state.clone() };
                return handler(ctx).await;
            }
        }

        let ctx = Ctx { req, state: self.inner.state.clone() };
        (self.inner.default_handler)(ctx).await
    }
}