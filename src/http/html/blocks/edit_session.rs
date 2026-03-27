use f1_bot_types::Session;

use crate::http::html::HtmlBuilder;
use std::fmt::Write;

impl HtmlBuilder {
    pub fn edit_session(mut self, session: &Session) -> Result<Self, std::fmt::Error> {
        write!(
            &mut self,
            r#"
<div class="dialog" id="edit-dialog">
<div class="col p-6 dialog-content"><form id="edit-event-form">
<input type="hidden" name="session_id" value="{session_id}" required />
<h1>Edit Event</h1>
<div class="form-group"><label for="event_title">Event Name</label>
<input type="text" name="name" id="event_title" required placeholder="Monaco Grand Prix" value="{event_name}"/></div>
<div class="form-group flex-1">
<label for="start_time">Start Time (Local time)</label>
<input type="datetime-local" data-utc="{start_time_utc}" required name="start_time" value="{start_time}" id="start_time"/>
</div>
<div class="row gap-1">
<div class="form-group">
<label>Status</label>
<div class="custom-select" value="{status}">
    <input type="hidden" name="status" value="{status}"/>
    <div class="active-value">{status}</div><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewbox="0 0 24 24" fill="none" stroke="currentcolor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down-icon lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
    <div class="options">
        <div class="option" data-value="Open">Open</div>
        <div class="option" data-value="Finished">Finished</div>
</div></div></div>
<div class="form-group">
<label>Status</label>
<div class="custom-select" value="{notify}">
    <input type="hidden" name="notify" value="{notify}"/>
    <div class="active-value">{notify}</div><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewbox="0 0 24 24" fill="none" stroke="currentcolor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down-icon lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
    <div class="options">
        <div class="option" data-value="Notify">Notify</div>
        <div class="option" data-value="Ignore">Ignore</div>
</div></div></div></div>
<div class="row flex-end gap-1 row-reverse"><button>Save</button>
<button id="cancel-edit-series" cancel class="reversed">Cancel</button>
</div>
</div>
</form></div></div>"#,
            session_id = session.id,
            event_name = session.name,
            status = session.status,
            notify = session.notify,
            start_time = session.start_time.format("%Y-%m-%dT%H:%M"),
            start_time_utc = session
                .start_time
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        )?;
        Ok(self)
    }
}
