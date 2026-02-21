use maud::{html, Markup};

pub fn ws_demo_page() -> Markup {
    html! {
        div class="container" {
            div class="card" {
                h1 { "WebSocket Demo" }
                p { "A persistent real-time connection managed via Tokio and Hyper-tungstenite." }
                
                div style="display: flex; gap: 1rem; align-items: stretch;" {
                    div style="flex: 1;" {
                        button id="ws-connect" class="btn" { "Connect" }
                        button id="ws-disconnect" class="btn btn-outline" style="margin-top: 0.5rem;" disabled { "Disconnect" }
                        button id="ws-ping" class="btn" style="margin-top: 1rem; width: 100%; display: block;" disabled { "Send Ping" }
                        button id="ws-inc" class="btn" style="margin-top: 0.5rem; width: 100%; display: block;" disabled { "Realtime Increment" }
                    }
                    div style="flex: 2; background: #1a1b26; border-radius: 8px; padding: 1rem; color: #a9b1d6; font-family: monospace; overflow-y: auto; max-height: 250px;" id="ws-logs" {
                        div { "> Disconnected" }
                    }
                }
            }

            div style="margin-top: 2rem;" {
                a href="/" class="nav-link" { "← Back to Home" }
            }
        }
    }
}

