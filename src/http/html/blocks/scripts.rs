use std::fmt::Write;

use crate::http::html::HtmlBuilder;

impl HtmlBuilder {
    pub fn scripts(mut self, list: &[&str]) -> Result<Self, std::fmt::Error> {
        for item in list {
            write!(
                &mut self,
                r#"<script src="/assets/js/{item}.js" defer></script>"#
            )?;
        }
        Ok(self)
    }
}
