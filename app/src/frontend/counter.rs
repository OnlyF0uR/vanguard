use maud::{html, Markup, PreEscaped};
use vanguard_core::state::serialize_state;
use serde::Serialize;

#[derive(Serialize)]
pub struct CounterState {
    pub count: i32,
}

pub fn counter_page(state: &CounterState) -> Markup {
    html! {
        div class="container" {
            div class="card" {
                h1 { "Counter Demo" }
                p { "This counter persists per-user using server-side storage and hydration." }
                
                div style="text-align: center;" {
                    span id="count-display" { (state.count) }
                    div {
                        button id="inc-btn" class="btn" { "Increment Count" }
                    }
                }
            }

            div style="margin-top: 2rem;" {
                a href="/" class="nav-link" { "← Back to Home" }
            }

            (serialize_state("counter", state))

            script data-page {
                (PreEscaped(r#"
                    const btn = document.getElementById('inc-btn');
                    const display = document.getElementById('count-display');
                    let state = Router.getState('counter') || { count: 0 };
                    display.innerText = state.count;

                    btn.onclick = async () => {
                        // Minimal visual feedback without changing text
                        btn.style.opacity = '0.7';
                        try {
                            const response = await fetch('/api/increment', { method: 'POST' });
                            if (response.ok) {
                                const data = await response.json();
                                Router.setState('counter', { count: data.count }, true);
                                display.innerText = data.count;
                            } else if (response.status === 401) {
                                window.location.href = '/login';
                            }
                        } catch (err) {
                            console.error('Increment failed:', err);
                        } finally {
                            btn.style.opacity = '1';
                        }
                    };
                "#))
            }
        }
    }
}