use maud::{html, Markup};

pub fn home_page() -> Markup {
    html! {
        div class="container" {
            h1 { "Welcome to Vanguard" }
            p { "This is a demonstration of a minimal, ultra-fast Rust fullstack framework." }
            
            div class="features" {
                h2 { "New: Authentication Support" }
                p { "Secure tokens powered by fn-dsa and cookies." }
                a href="/login" class="btn" { "Go to Login" }
            }

            a href="/counter" class="btn" { "Try the Counter ->" }
        }
    }
}
