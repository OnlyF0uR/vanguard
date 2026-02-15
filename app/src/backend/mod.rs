use vanguard_auth::AuthManager;
use dashmap::DashMap;

pub struct AppState {
    pub auth: AuthManager,
    pub secure: bool,
    pub counters: DashMap<String, i32>,
}

pub mod handlers;