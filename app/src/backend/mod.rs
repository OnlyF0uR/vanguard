use vanguard_auth::AuthManager;
use dashmap::DashMap;
use vanguard_core::rate_limit::RateLimiter;

pub struct AppState {
    pub auth: AuthManager,
    pub secure: bool,
    pub counters: DashMap<String, i32>,
    pub rate_limiter: RateLimiter,
}

pub mod handlers;
pub mod ws_handlers;