use maud::{html, Markup, PreEscaped, DOCTYPE};

pub struct Page {
    title: String,
    description: Option<String>,
    keywords: Vec<String>,
    canonical_url: Option<String>,
    og_image: Option<String>,
    head_tags: Vec<Markup>,
    inline_css: Vec<String>,
    inline_js: Vec<String>,
    content: Option<Markup>,
}

impl Page {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            keywords: Vec::new(),
            canonical_url: None,
            og_image: None,
            head_tags: Vec::new(),
            inline_css: Vec::new(),
            inline_js: Vec::new(),
            content: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn keywords(mut self, keywords: &[&str]) -> Self {
        self.keywords = keywords.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn canonical_url(mut self, url: impl Into<String>) -> Self {
        self.canonical_url = Some(url.into());
        self
    }

    pub fn og_image(mut self, img: impl Into<String>) -> Self {
        self.og_image = Some(img.into());
        self
    }

    pub fn head(mut self, tag: Markup) -> Self {
        self.head_tags.push(tag);
        self
    }

    pub fn inline_css(mut self, css: impl Into<String>) -> Self {
        self.inline_css.push(css.into());
        self
    }

    pub fn inline_js(mut self, js: impl Into<String>) -> Self {
        self.inline_js.push(js.into());
        self
    }

    pub fn content(mut self, content: Markup) -> Self {
        self.content = Some(content);
        self
    }

    pub fn render(self) -> Markup {
        html! {
            (DOCTYPE)
            html {
                head {
                    meta charset="utf-8";
                    meta name="viewport" content="width=device-width, initial-scale=1.0";
                    title { (self.title) }

                    @if let Some(desc) = &self.description {
                        meta name="description" content=(desc);
                    }

                    @if !self.keywords.is_empty() {
                        meta name="keywords" content=(self.keywords.join(", "));
                    }

                    @if let Some(url) = &self.canonical_url {
                        link rel="canonical" href=(url);
                    }

                    @if let Some(img) = &self.og_image {
                        meta property="og:image" content=(img);
                        meta name="twitter:card" content="summary_large_image";
                    }

                    link rel="stylesheet" href="/static/app.css";
                    script src="/static/router.js" {}

                    @for tag in self.head_tags {
                        (tag)
                    }

                    @for css in self.inline_css {
                        style { (PreEscaped(css)) }
                    }

                    @for js in self.inline_js {
                        script data-page { (PreEscaped(js)) }
                    }
                }
                body {
                    div id="app" {
                        @if let Some(content) = self.content {
                            (content)
                        }
                    }
                }
            }
        }
    }
}