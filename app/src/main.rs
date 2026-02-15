mod backend;
mod frontend;

use backend::AppState;
use backend::handlers::*;
use vanguard_core::router::Router;
use vanguard_core::server::{self, ServerConfig};
use vanguard_core::static_files::static_handler;
use vanguard_auth::AuthManager;
use std::net::SocketAddr;
use std::env;
use dashmap::DashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _ = dotenvy::dotenv();

    let auth = AuthManager::new().await?;
    let secure = env::var("ENABLE_SSH").map(|v| v == "true").unwrap_or(false);
    let cors_domain = env::var("CORS_DOMAIN").ok();
    
    let state = AppState { 
        auth, 
        secure,
        counters: DashMap::new(),
    };

    let router = Router::new(state)
        .get("/", home_handler)
        .get("/counter", counter_handler)
        .get("/login", login_get_handler)
        .post("/login", login_post_handler)
        .post("/logout", logout_handler)
        .get("/profile", profile_handler)
        .post("/api/increment", increment_api_handler) // New API route
        .mount("/static", static_handler("."));

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse().unwrap_or(3000);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    
    let config = ServerConfig {
        addr,
        cors_domain,
    };

    server::run(config, router).await
}