use f1_bot_types::Weekend;

use crate::http::html::HtmlBuilder;
use std::fmt::Write;

impl HtmlBuilder {
    pub fn edit_event(mut self, weekend: &Weekend) -> Result<Self, std::fmt::Error> {
        write!(
            &mut self,
            r#"
<div class="dialog" id="edit-dialog">
<div class="col p-6 dialog-content"><form id="edit-event-form">
<input type="hidden" name="event_id" value="{event_id}" required />
<h1>Edit Event</h1>
<div class="form-group"><label for="event_title">Event Name</label>
<input type="text" name="name" id="event_title" required placeholder="Monaco Grand Prix" value="{event_name}"/></div>
<div class="row gap-1">
<div class="form-group flex-1"><label for="icon">Icon (country code)</label>
<input type="text" name="icon" minlength="2" required maxlength="2" id="icon" placeholder="MC" value="{country_code}"/></div>
<div class="form-group"><label>Series</label>
<div class="custom-select" value="{series}">
    <input type="hidden" name="series" value="{series}"/>
    <div class="active-value">{series}</div><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down-icon lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
    <div class="options">
        <div class="option" data-value="F1">Formula 1</div>
        <div class="option" data-value="F2">Formula 2</div>
        <div class="option" data-value="F3">Formula 3</div>
        <div class="option" data-value="F1Academy">F1 Academy</div>
</div></div></div></div>
<div class="row gap-1"><div class="form-group flex-1">
<label for="start_date">Start Date</label>
<input type="date" required name="start_date" value="{start_date}" id="start_date"/>
</div>
<div class="form-group">
<label>Status</label>
<div class="custom-select" value="{status}">
    <input type="hidden" name="status" value="{status}"/>
    <div class="active-value">{status}</div><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down-icon lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
    <div class="options">
        <div class="option" data-value="Open">Open</div>
        <div class="option" data-value="Done">Done</div>
</div></div></div>
</div>
<div class="row flex-end gap-1">
    <button id="cancel-new-event" cancel class="reversed">Cancel</button><button>Save</button>
</div>
</div>
</form></div></div>"#,
            event_id = weekend.id,
            event_name = weekend.name,
            country_code = &weekend.icon[6..8],
            series = weekend.series,
            status = weekend.status,
            start_date = weekend.start_date.format("%Y-%m-%d"),
        )?;
        Ok(self)
    }
}
