use crate::http::html::HtmlBuilder;
use std::fmt::Write;

impl HtmlBuilder {
    pub fn main<F>(mut self, classes: Option<&str>, content: F) -> Result<Self, std::fmt::Error>
    where
        F: FnOnce(HtmlBuilder) -> Result<HtmlBuilder, std::fmt::Error>,
    {
        write!(
            &mut self,
            r#"<main class="{}">"#,
            classes.unwrap_or_default()
        )?;
        let mut s = content(self)?;
        s.write_str("</main>")?;
        Ok(s)
    }
}
