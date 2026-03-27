use f1_bot_types::Weekend;

use crate::http::html::HtmlBuilder;
use std::fmt::Write;

impl HtmlBuilder {
    pub fn session_list<T, F>(
        mut self,
        event: &Weekend,
        list: &[T],
        for_session: F,
    ) -> Result<Self, std::fmt::Error>
    where
        T: Sized,
        F: Fn(&mut HtmlBuilder, &T) -> Result<(), std::fmt::Error>,
    {
        write!(
            &mut self,
            r#"<div class="main-list flex-1 sessions" id="sessions-container">
<div class="container-header col gap-3">
<div class="row space-between align-center">
<div class="col">
<div class="card-title align-center row"><svg class="session-header-svg" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-clock-icon lucide-clock"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
Sessions
</div>
<span class="session-ev-title">{series} {name}</span>
</div>
<button id="add-session"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-plus-icon lucide-plus"><path d="M5 12h14"/><path d="M12 5v14"/></svg> Add Session</button>
</div></div><div class="overflow-y col">"#,
            series = event.series,
            name = event.name
        )?;

        if list.is_empty() {
            write!(
                &mut self,
                r#"<span class="empty-session">No sessions for this event</span>"#
            )?;
        }

        for item in list {
            for_session(&mut self, item)?;
        }

        self.write_str("</div></div>")?;

        Ok(self)
    }

    pub fn empty_sessions(mut self) -> Result<Self, std::fmt::Error> {
        write!(
            &mut self,
            r#"<div class="main-list flex-1 sessions" id="sessions-container"><div class="no-session">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-clock-icon lucide-clock"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
            <span>Select an event to view its sessions</span>
            </div></div>"#
        )?;
        Ok(self)
    }
}
