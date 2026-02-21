use hyper::{Response, StatusCode};
use http_body_util::{Full, BodyExt};
use bytes::Bytes;
use vanguard_core::router::{HandlerResponse, Ctx};
use vanguard_core::view::Page;
use vanguard_auth::Claims;
use crate::backend::AppState;
use crate::frontend::{home, counter, auth};
use crate::frontend::counter::CounterState;
use serde::Deserialize;
use validator::Validate;

pub async fn home_handler(ctx: Ctx<AppState>) -> HandlerResponse {
    let is_auth = get_user(&ctx).is_some();
    let content = home::home_page(is_auth);
    let page = Page::new("Home - Vanguard")
        .description("A high-performance Rust foundation for secure, server-rendered web applications.")
        .keywords(&["vanguard", "rust", "framework", "hyper", "web"])
        .content(content)
        .render()
        .into_string();
    Ok(html_response(page))
}

pub async fn counter_handler(ctx: Ctx<AppState>) -> HandlerResponse {
    let user = match get_user(&ctx) {
        Some(u) => u,
        None => return redirect("/login"),
    };

    let count = *ctx.state.counters.entry(user.sub.clone()).or_insert(0);
    
    let state = CounterState { count };
    let content = counter::counter_page(&state);
    let page = Page::new("Counter - Vanguard")
        .description("Interactive counter demo built with Vanguard.")
        .content(content)
        .render()
        .into_string();
    Ok(html_response(page))
}

pub async fn increment_api_handler(ctx: Ctx<AppState>) -> HandlerResponse {
    if !ctx.state.rate_limiter.check_limit(ctx.ip()) {
        return Ok(Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .header("Content-Type", "text/plain")
            .body(Full::new(Bytes::from("429 Too Many Requests: Slow down!")))
            .unwrap());
    }

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

    let csrf_token = ctx.state.auth.generate_csrf_token();
    let csrf_cookie = ctx.state.auth.csrf_cookie(&csrf_token, ctx.state.secure);

    let content = auth::login_page(None, &csrf_token);
    let page = Page::new("Login - Vanguard")
        .description("Log in to your Vanguard account.")
        .content(content)
        .render()
        .into_string();

    Ok(Response::builder()
        .header("Content-Type", "text/html")
        .header("Set-Cookie", csrf_cookie)
        .body(Full::new(Bytes::from(page)))
        .unwrap())
}

#[derive(Deserialize, Validate)]
struct LoginData {
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    username: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    #[allow(dead_code)]
    password: String,
    csrf_token: String,
}

pub async fn login_post_handler(mut ctx: Ctx<AppState>) -> HandlerResponse {
    let body_bytes = ctx.req.body_mut().collect().await?.to_bytes();
    let login_data: LoginData = match serde_urlencoded::from_bytes(&body_bytes) {
        Ok(data) => data,
        Err(_) => {
            let page = Page::new("Login - Vanguard")
                .content(auth::login_page(Some("Invalid form data"), ""))
                .render()
                .into_string();
            return Ok(html_response(page));
        }
    };

    if let Err(e) = login_data.validate() {
        let err_msg = e.to_string();
        let page = Page::new("Login - Vanguard")
            .content(auth::login_page(Some(&err_msg), ""))
            .render()
            .into_string();
        return Ok(html_response(page));
    }

    let cookie_token = ctx.cookie("vanguard_csrf");
    if cookie_token.is_none() || cookie_token.unwrap() != login_data.csrf_token {
        let page = Page::new("Login - Vanguard")
            .content(auth::login_page(Some("Invalid CSRF token"), ""))
            .render()
            .into_string();
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("Content-Type", "text/html")
            .body(Full::new(Bytes::from(page)))
            .unwrap());
    }

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
        Err(_) => {
            let page = Page::new("Login - Vanguard")
                .content(auth::login_page(Some("Internal error generating token"), ""))
                .render()
                .into_string();
            Ok(html_response(page))
        }
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
            let page = Page::new(format!("Profile - {}", claims.sub))
                .description("User profile and settings.")
                .content(content)
                .render()
                .into_string();
            Ok(html_response(page))
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

pub fn get_user(ctx: &Ctx<AppState>) -> Option<Claims> {
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