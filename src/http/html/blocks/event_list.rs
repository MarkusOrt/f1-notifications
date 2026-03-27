use crate::http::html::HtmlBuilder;
use std::fmt::Write;

impl HtmlBuilder {
    pub fn event_list<T, F>(mut self, list: &[T], for_weekend: F) -> Result<Self, std::fmt::Error>
    where
        T: Sized,
        F: Fn(&mut HtmlBuilder, &T) -> Result<(), std::fmt::Error>,
    {
        write!(
            &mut self,
            r#"<div class="main-list flex-1">
<div class="container-header col gap-3">
<div class="row space-between">
<div class="card-title align-center row"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-calendar-icon lucide-calendar"><path d="M8 2v4"/><path d="M16 2v4"/><rect width="18" height="18" x="3" y="4" rx="2"/><path d="M3 10h18"/></svg>
Events
</div>
<button id="add-event"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus-icon lucide-plus"><path d="M5 12h14"/><path d="M12 5v14"/></svg> Add Event</button>
</div><div><div class="filters row align-center space-between"><div class="row align-center gap-5">
<svg class="funnel" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-funnel-icon lucide-funnel"><path d="M10 20a1 1 0 0 0 .553.895l2 1A1 1 0 0 0 14 21v-7a2 2 0 0 1 .517-1.341L21.74 4.67A1 1 0 0 0 21 3H3a1 1 0 0 0-.742 1.67l7.225 7.989A2 2 0 0 1 10 14z"/></svg>
<div class="custom-select" id="series-filter" value="all">
    <input type="hidden" value="all" />
    <div class="active-value">All Series</div><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down-icon lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
    <div class="options">
        <div class="option" data-value="all">All Series</div>
        <div class="option" data-value="F1">Formula 1</div>
        <div class="option" data-value="F2">Formula 2</div>
        <div class="option" data-value="F3">Formula 3</div>
        <div class="option" data-value="F1Academy">F1 Academy</div>
    </div>
</div>
<div id="status-filter" class="custom-select" value="all">
    <input type="hidden" value="all" />
    <div class="active-value">All</div><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down-icon lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
    <div class="options">
        <div class="option" data-value="all">All</div>
        <div class="option" data-value="Open">Open</div>
        <div class="option" data-value="Done">Done</div>
    </div>
</div>
</div></div></div></div><div class="overflow-y">"#
        )?;

        for item in list {
            for_weekend(&mut self, item)?;
        }

        self.write_str("</div></div>")?;

        Ok(self)
    }
}
