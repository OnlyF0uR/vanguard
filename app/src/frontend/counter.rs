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
            h1 { "Interactive Counter" }
            p { "This counter is persistent and synced to your user account." }
            
            div class="counter-box" {
                p { "Current count: " span id="count-display" { (state.count) } }
                button id="inc-btn" class="btn" { "Increment (API)" }
            }

            (serialize_state("counter", state))

            script data-page {
                (PreEscaped(r#"
                    const btn = document.getElementById('inc-btn');
                    const display = document.getElementById('count-display');
                    let state = Router.getState('counter') || { count: 0 };
                    display.innerText = state.count;

                    btn.onclick = async () => {
                        btn.disabled = true;
                        try {
                            const response = await fetch('/api/increment', { method: 'POST' });
                            if (response.ok) {
                                const data = await response.json();
                                state.count = data.count;
                                Router.setState('counter', state, true); // Force update current session state
                                Router.invalidateCache('/counter'); // Invalidate stale HTML cache
                                display.innerText = state.count;
                            } else if (response.status === 401) {
                                window.location.href = '/login';
                            }
                        } catch (err) {
                            console.error('Failed to increment:', err);
                        } finally {
                            btn.disabled = false;
                        }
                    };
                "#))
            }

            p { a href="/" { "<- Back to Home" } }
        }
    }
}