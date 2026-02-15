use serde::Serialize;
use maud::{html, Markup, PreEscaped};

/// Serializes state for the current page.
/// Uses override=false by default so that stale cached SSR state 
/// doesn't overwrite more recent client-side updates during SPA navigation.
pub fn serialize_state<T: Serialize>(key: &str, value: &T) -> Markup {
    let json = serde_json::to_string(value).unwrap_or_default();
    html! {
        script data-page {
            (PreEscaped(format!("Router.setState('{}', {}, false);", key, json)))
        }
    }
}

/// Serializes state accessible globally.
pub fn serialize_global_state<T: Serialize>(key: &str, value: &T) -> Markup {
    let json = serde_json::to_string(value).unwrap_or_default();
    html! {
        script data-page {
            (PreEscaped(format!("Router.setGlobalState('{}', {}, false);", key, json)))
        }
    }
}
