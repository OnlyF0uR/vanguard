use maud::{html, Markup};

pub fn login_page(error: Option<&str>) -> Markup {
    html! {
        div class="container" {
            h1 { "Login" }
            
            @if let Some(err) = error {
                div class="error" { (err) }
            }

            form action="/login" method="POST" class="auth-form" {
                div {
                    label { "Username: " }
                    input type="text" name="username" required;
                }
                div {
                    label { "Password: " }
                    input type="password" name="password" required;
                }
                button type="submit" class="btn" { "Sign In" }
            }

            p { "Hint: Use any username and password." }
            p { a href="/" { "<- Back to Home" } }
        }
    }
}

pub fn profile_page(username: &str) -> Markup {
    html! {
        div class="container" {
            h1 { "Profile" }
            p { "Welcome back, " b { (username) } "!" }
            
            form action="/logout" method="POST" {
                button type="submit" class="btn" { "Logout" }
            }
            
            p { a href="/" { "<- Back to Home" } }
        }
    }
}
