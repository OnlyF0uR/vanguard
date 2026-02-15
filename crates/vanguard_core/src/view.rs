use maud::{html, Markup, DOCTYPE};

pub fn base_layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                link rel="stylesheet" href="/static/app.css";
                script src="/static/router.js" {}
            }
            body {
                div id="app" {
                    (content)
                }
            }
        }
    }
}