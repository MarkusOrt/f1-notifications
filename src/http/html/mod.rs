use axum::response::Html;

pub mod blocks {
    pub mod add_event;
    pub mod add_session;
    pub mod edit_event;
    pub mod edit_session;
    pub mod event_list;
    pub mod footer;
    pub mod head;
    pub mod header;
    pub mod main;
    pub mod scripts;
    pub mod session_list;
}

pub struct HtmlBuilder(String);

impl HtmlBuilder {
    pub fn new() -> Self {
        HtmlBuilder(String::new())
    }

    pub fn with_capacity(cap: usize) -> Self {
        HtmlBuilder(String::with_capacity(cap))
    }
}

impl std::fmt::Write for HtmlBuilder {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.write_str(s)
    }
}

impl From<HtmlBuilder> for Html<String> {
    fn from(value: HtmlBuilder) -> Self {
        Html(value.0)
    }
}
