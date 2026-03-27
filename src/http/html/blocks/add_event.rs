use crate::http::html::HtmlBuilder;
use std::fmt::Write;

impl HtmlBuilder {
    pub fn add_event(mut self) -> Result<Self, std::fmt::Error> {
        write!(
            &mut self,
            r#"<div id="backdrop" class="backdrop"></div>
<div class="dialog" id="add-event-dialog">
<div class="col p-6 dialog-content"><form id="create-new-event-form">
<h1>Add new Event</h1>
<div class="form-group"><label for="event_title">Event Name</label>
<input type="text" name="name" required id="event_title" placeholder="Monaco Grand Prix"/></div>
<div class="row gap-1">
<div class="form-group flex-1"><label for="icon">Icon (country code)</label>
<input type="text" name="icon" minlength="2" maxlength="2" required id="icon" placeholder="MC"/></div>
<div class="form-group"><label>Series</label>
<div class="custom-select" value="Formula 1">
    <input type="hidden" name="series" value="F1"/>
    <div class="active-value">Formula 1</div><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down-icon lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
    <div class="options">
        <div class="option" data-value="F1">Formula 1</div>
        <div class="option" data-value="F2">Formula 2</div>
        <div class="option" data-value="F3">Formula 3</div>
        <div class="option" data-value="F1Academy">F1 Academy</div>
</div></div></div></div>
<div class="row gap-1"><div class="form-group flex-1">
<label for="start_date">Start Date</label>
<input type="date" name="start_date" required id="start_date"/>
</div>
<div class="form-group">
<label>Status</label>
<div class="custom-select" value="Open">
    <input type="hidden" name="status" value="Open"/>
    <div class="active-value">Open</div><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down-icon lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
    <div class="options">
        <div class="option" data-value="Open">Open</div>
        <div class="option" data-value="Done">Done</div>
</div></div></div>
</div>
<div class="row flex-end gap-1">
    <button id="cancel-new-event" cancel class="reversed">Cancel</button><button>Add Event</button>
</div>
</div>
</form></div></div>"#
        )?;
        Ok(self)
    }
}
