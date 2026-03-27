use crate::http::html::HtmlBuilder;
use std::fmt::Write;

impl HtmlBuilder {
    pub fn add_session(mut self) -> Result<Self, std::fmt::Error> {
        write!(
            &mut self,
            r#"
<div class="dialog" id="add-session-dialog">
<div class="col p-6 dialog-content"><form id="edit-event-form">
<input type="hidden" name="event_id" required />
<h1>Add new Session</h1>
<div class="form-group"><label for="session_title">Session Name</label>
<input type="text" name="name" id="session_title" required placeholder="Free Practice 1"/></div>
<div class="form-group flex-1">
<label for="start_time">Start Time (Local time)</label>
<input type="datetime-local" required name="start_time" id="start_time"/>
</div>
<div class="row gap-1">
<div class="form-group">
<label>Status</label>
<div class="custom-select" value="Open">
    <input type="hidden" name="status" value="Open"/>
    <div class="active-value">Open</div><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewbox="0 0 24 24" fill="none" stroke="currentcolor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down-icon lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
    <div class="options">
        <div class="option" data-value="Open">Open</div>
        <div class="option" data-value="Finished">Finished</div>
</div></div></div>
<div class="form-group">
<label>Status</label>
<div class="custom-select" value="Notify">
    <input type="hidden" name="notify" value="Notify"/>
    <div class="active-value">Notify</div><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewbox="0 0 24 24" fill="none" stroke="currentcolor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-chevron-down-icon lucide-chevron-down"><path d="m6 9 6 6 6-6"/></svg>
    <div class="options">
        <div class="option" data-value="Notify">Notify</div>
        <div class="option" data-value="Ignore">Ignore</div>
</div></div></div></div>
<div class="row flex-end gap-1 row-reverse"><button>Save</button>
<button id="cancel-edit-series" cancel class="reversed">Cancel</button>
</div>
</div>
</form></div></div>"#
        )?;
        Ok(self)
    }
}
