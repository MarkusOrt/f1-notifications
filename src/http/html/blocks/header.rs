use crate::http::html::HtmlBuilder;
use std::fmt::Write;

impl HtmlBuilder {
    pub fn header(mut self) -> Result<Self, std::fmt::Error> {
        write!(
            &mut self,
            r#"<header>
<div class="content-width row sticky">
<div class="row gap-1">
<div class="logo">
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-flag-icon lucide-flag"><path d="M4 22V4a1 1 0 0 1 .4-.8A6 6 0 0 1 8 2c3 0 5 2 7.333 2q2 0 3.067-.8A1 1 0 0 1 20 4v10a1 1 0 0 1-.4.8A6 6 0 0 1 16 16c-3 0-5-2-8-2a6 6 0 0 0-4 1.528"/></svg>
</div>
<div class="col">
<h1>F1 Notifications</h1>
<p>Manage race events & notifications</p>
</div>
</div>
</div></header>"#
        )?;
        Ok(self)
    }
}
