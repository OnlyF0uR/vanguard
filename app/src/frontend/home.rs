use maud::{html, Markup};

pub fn home_page() -> Markup {
    html! {
        div {
            section style="margin-bottom: 4rem;" {
                h1 { "Vanguard Framework" }
                p { "A high-performance Rust foundation for secure, server-rendered web applications." }
                div style="display: flex; gap: 1rem;" {
                    a href="/login" class="btn" { "Get Started" }
                    a href="/counter" class="btn btn-outline" { "View Demo" }
                }
            }
            
            div class="grid" {
                div class="grid-item" {
                    h3 { "Performance" }
                    p { "Built on Hyper 1.0 for low-latency, high-throughput request handling." }
                }
                div class="grid-item" {
                    h3 { "Security" }
                    p { "Session integrity verified with 512-bit FN-DSA digital signatures." }
                }
                div class="grid-item" {
                    h3 { "Architecture" }
                    p { "Clean separation between server-side logic and modular frontend components." }
                }
            }
        }
    }
}
