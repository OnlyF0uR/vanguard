use hyper::{Response, StatusCode};
use http_body_util::{Full, BodyExt};
use bytes::Bytes;
use vanguard_core::router::{HandlerResponse, Ctx};
use vanguard_core::view::base_layout;
use vanguard_auth::Claims;
use crate::backend::AppState;
use crate::frontend::{home, counter, auth};
use crate::frontend::counter::CounterState;
use serde::Deserialize;

pub async fn home_handler(_ctx: Ctx<AppState>) -> HandlerResponse {
    let content = home::home_page();
    Ok(html_response(base_layout("Home - Vanguard", content).into_string()))
}

pub async fn counter_handler(ctx: Ctx<AppState>) -> HandlerResponse {
    let user = match get_user(&ctx) {
        Some(u) => u,
        None => return redirect("/login"),
    };

    let count = *ctx.state.counters.entry(user.sub.clone()).or_insert(0);
    
    let state = CounterState { count };
    let content = counter::counter_page(&state);
    Ok(html_response(base_layout("Counter - Vanguard", content).into_string()))
}

pub async fn increment_api_handler(ctx: Ctx<AppState>) -> HandlerResponse {
    let user = match get_user(&ctx) {
        Some(u) => u,
        None => return Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body(Full::new(Bytes::from("Unauthorized"))).unwrap()),
    };

    let mut count = ctx.state.counters.entry(user.sub).or_insert(0);
    *count += 1;
    let new_count = *count;

    let json = serde_json::json!({ "success": true, "count": new_count }).to_string();
    
    Ok(Response::builder()
        .header("Content-Type", "application/json")
        .body(Full::new(Bytes::from(json)))
        .unwrap())
}

pub async fn login_get_handler(ctx: Ctx<AppState>) -> HandlerResponse {
    if get_user(&ctx).is_some() {
        return redirect("/profile");
    }
    let content = auth::login_page(None);
    Ok(html_response(base_layout("Login - Vanguard", content).into_string()))
}

#[derive(Deserialize)]
struct LoginData {
    username: String,
    #[allow(dead_code)]
    password: String,
}

pub async fn login_post_handler(mut ctx: Ctx<AppState>) -> HandlerResponse {
    let body_bytes = ctx.req.body_mut().collect().await?.to_bytes();
    let login_data: LoginData = match serde_urlencoded::from_bytes(&body_bytes) {
        Ok(data) => data,
        Err(_) => return Ok(html_response(base_layout("Login - Vanguard", auth::login_page(Some("Invalid form data"))).into_string())),
    };

    match ctx.state.auth.create_token(&login_data.username, 60) {
        Ok(token) => {
            let cookie = ctx.state.auth.auth_cookie(token, ctx.state.secure);
            Ok(Response::builder()
                .status(StatusCode::SEE_OTHER)
                .header("Location", "/profile")
                .header("Set-Cookie", cookie)
                .body(Full::new(Bytes::new()))
                .unwrap())
        }
        Err(_) => Ok(html_response(base_layout("Login - Vanguard", auth::login_page(Some("Internal error generating token"))).into_string())),
    }
}

pub async fn logout_handler(ctx: Ctx<AppState>) -> HandlerResponse {
    let mut cookie = ctx.state.auth.auth_cookie("".to_string(), ctx.state.secure);
    cookie.push_str("; Max-Age=0");

    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", "/")
        .header("Set-Cookie", cookie)
        .body(Full::new(Bytes::new()))
        .unwrap())
}

pub async fn profile_handler(ctx: Ctx<AppState>) -> HandlerResponse {
    match get_user(&ctx) {
        Some(claims) => {
            let content = auth::profile_page(&claims.sub);
            Ok(html_response(base_layout("Profile - Vanguard", content).into_string()))
        }
        None => redirect("/login"),
    }
}

// Helpers
fn html_response(html: String) -> Response<Full<Bytes>> {
    Response::builder()
        .header("Content-Type", "text/html")
        .body(Full::new(Bytes::from(html)))
        .unwrap()
}

fn redirect(path: &str) -> HandlerResponse {
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header("Location", path)
        .body(Full::new(Bytes::new()))
        .unwrap())
}

fn get_user(ctx: &Ctx<AppState>) -> Option<Claims> {
    let cookie_header = ctx.req.headers().get("Cookie")?.to_str().ok()?;
    for cookie_str in cookie_header.split(';') {
        let cookie_str = cookie_str.trim();
        if cookie_str.starts_with("vanguard_auth=") {
            let token = &cookie_str["vanguard_auth=".len()..];
            return ctx.state.auth.verify_token(token);
        }
    }
    None
}