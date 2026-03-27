use std::fmt::Write;

use crate::http::html::HtmlBuilder;

impl HtmlBuilder {
    pub fn head(mut self, title: &str) -> Result<Self, std::fmt::Error> {
        write!(
            &mut self,
            r#"<!doctype html><html lang="en">
<head><title>F1 Notifications Bot | {}</title>
<meta charset="UTF-8"/>
<meta name="generator" value="F1-Notifications"/>
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<link href="/assets/style.css" rel="stylesheet" media="screen" />
</head>"#,
            title
        )?;
        Ok(self)
    }
}
