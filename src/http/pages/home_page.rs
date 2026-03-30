use std::fmt::Write;

use axum::{
    extract::State,
    response::{Html, Redirect},
};
use f1_bot_types::Weekend;

use crate::{
    bot::database,
    http::{AxumState, auth::User, html::HtmlBuilder},
};

pub async fn get(
    user: Option<User>,
    State(app_state): State<AxumState<'_>>,
) -> Result<Result<Html<String>, Redirect>, crate::error::Error> {
    if user.is_none() {
        return Ok(Err(Redirect::temporary("/auth")));
    };
    let weekends = database::all_weekends(&app_state.db_pool).await?;
    Ok(Ok(HtmlBuilder::new()
        .head("Home")?
        .header()?
        .main(Some("row gap-2 flex-1 content-width"), |b| {
            b.event_list(&weekends, weekend_format)?.empty_sessions()
        })?
        .add_event()?
        .add_session()?
        .scripts(&["dash"])?
        .footer()?
        .into()))
}

pub fn weekend_format(html: &mut HtmlBuilder, item: &Weekend) -> Result<(), std::fmt::Error> {
    let mut country_code = match item.icon.len() {
        0..9 => "__",
        9.. => &item.icon[6..8],
    };
    if !country_code.is_ascii() {
        country_code = "__";
    }
    write!(
        html,
        r#"<div class="event row px-4 py-3" data-id="{id}" data-series="{series}" data-status="{status}">
<div class="cc">{country_code}</div>
<div class="col details flex-1"><div class="title">{title}</div>
<div class="row meta align-center gap-5"><span class="series-display series-{series}">{series}</span><span class="date-display" data-date="{date_utc}" data-dateformat="dmy">{date_display}</span></div>
</div>
<span class="status status-{status}"><span class="dot"></span>{status}</span>
<button class="actions" data-actions=""><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-ellipsis-icon lucide-ellipsis"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg></button>
<div class="actions-display">
    <button class="row" data-action="edit_event"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-pencil-icon lucide-pencil"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/></svg>Edit</button>
    <button class="destructive row" data-action="delete_event"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-trash2-icon lucide-trash-2"><path d="M10 11v6"/><path d="M14 11v6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>Delete</button>
</div>
</div>"#,
        id = item.id,
        country_code = country_code,
        title = item.name,
        series = item.series,
        date_utc = item
            .start_date
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        date_display = item.start_date.format("%d-%m-%Y"),
        status = item.status
    )?;
    Ok(())
}
