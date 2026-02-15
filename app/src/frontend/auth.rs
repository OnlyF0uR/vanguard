use maud::{html, Markup};

pub fn login_page(error: Option<&str>) -> Markup {
    html! {
        div class="container" {
            div class="card" {
                h1 { "Sign In" }
                p { "Enter your credentials to access your protected dashboard. Try 'username' and 'password'." }

                @if let Some(err) = error {
                    div class="error-banner" { (err) }
                }

                form action="/login" method="POST" {
                    div class="form-group" {
                        label { "Username" }
                        input type="text" name="username" required autocomplete="username";
                    }
                    div class="form-group" {
                        label { "Password" }
                        input type="password" name="password" required autocomplete="current-password";
                    }
                    button type="submit" class="btn" style="width: 100%; margin-top: 1rem;" { "Sign In" }
                }
            }

            div style="margin-top: 2rem;" {
                a href="/" class="nav-link" { "← Back to Home" }
            }
        }
    }
}

pub fn profile_page(username: &str) -> Markup {
    html! {
        div class="container" {
            div class="card" {
                h1 { "User Profile" }
                p { "Logged in as " b { (username) } }

                div style="padding: 1.5rem 0; border-top: 1px solid var(--border); margin-top: 1.5rem;" {
                    h3 style="font-size: 1rem; margin-bottom: 0.5rem;" { "Session Information" }
                    p style="font-size: 0.875rem;" { "Authentication is achieved using FN-DSA signatures." }
                }

                form action="/logout" method="POST" {
                    button type="submit" class="btn btn-outline" { "Sign Out" }
                }
            }

            div style="margin-top: 2rem;" {
                a href="/" class="nav-link" { "← Back to Home" }
            }
        }
    }
}
