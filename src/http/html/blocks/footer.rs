use std::fmt::Write;

use chrono::{Datelike, Utc};

use crate::http::html::HtmlBuilder;

impl HtmlBuilder {
    pub fn footer(mut self) -> Result<Self, std::fmt::Error> {
        write!(
            &mut self,
            r#"<footer>&copy; {} Markus Ort - All Rights Reserved.</footer></body></html>"#,
            Utc::now().year()
        )?;
        Ok(self)
    }
}
