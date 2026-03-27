use std::fmt::Write;

use axum::{
    extract::{Path, State},
    response::Html,
};
use f1_bot_types::{Session, SessionNotifySettings};

use crate::{
    bot::database,
    http::{AxumState, auth::User, html::HtmlBuilder},
};

pub async fn edit_dialog(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Path(session_id): Path<u64>,
) -> Result<Html<String>, crate::error::Error> {
    _ = user;
    let Some(session) = database::session(&app_state.db_pool, session_id).await? else {
        return Err(crate::error::Error::NotFound);
    };
    let html = HtmlBuilder::with_capacity(1024);
    Ok(html.edit_session(&session)?.into())
}

pub async fn get(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Path(event_id): Path<u64>,
) -> Result<Html<String>, crate::error::Error> {
    _ = user;
    let sessions = database::sessions_for_weekend_notx(&app_state.db_pool, event_id).await?;
    let weekend = database::weekend(&app_state.db_pool, event_id).await?;
    let html = HtmlBuilder::with_capacity(1024);
    Ok(html.session_list(&weekend, &sessions, per_session)?.into())
}

fn per_session(html: &mut HtmlBuilder, item: &Session) -> Result<(), std::fmt::Error> {
    write!(
        html,
        r#"<div class="session row px-4 py-3" data-id="{id}"><div class="notify-setting notify-{notify}">"#,
        id = item.id,
        notify = item.notify
    )?;
    match item.notify {
        f1_bot_types::SessionNotifySettings::Notify => html.write_str(r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-bell-icon lucide-bell"><path d="M10.268 21a2 2 0 0 0 3.464 0"/><path d="M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326"/></svg>"#)?,
        f1_bot_types::SessionNotifySettings::Ignore => html.write_str(r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-bell-off-icon lucide-bell-off"><path d="M10.268 21a2 2 0 0 0 3.464 0"/><path d="M17 17H4a1 1 0 0 1-.74-1.673C4.59 13.956 6 12.499 6 8a6 6 0 0 1 .258-1.742"/><path d="m2 2 20 20"/><path d="M8.668 3.01A6 6 0 0 1 18 8c0 2.687.77 4.653 1.707 6.05"/></svg>"#)?,
    };
    write!(
        html,
        r#"</div><div class="col flex-1">
        <div class="title">{title}</div>
        <div class="datetime" data-time="{start}">{start}</div>
        </div>
        <span class="status status-{status}"><span class="dot"></span>{status}</span>
        <button class="actions" data-actions=""><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-ellipsis-icon lucide-ellipsis"><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/><circle cx="5" cy="12" r="1"/></svg></button>
        <div class="actions-display">
<button data-id="{id}" data-action="edit_session"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-pencil-icon lucide-pencil"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/></svg>Edit</button>
<button data-action="{toggle_action}" data-id="{id}">{notify_text}</button>
<div class="separator-h"></div>
<button class="destructive" data-id="{id}" data-action="delete-session"><svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-trash2-icon lucide-trash-2"><path d="M10 11v6"/><path d="M14 11v6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>Delete</button>
        </div>
        </div>"#,
        id = item.id,
        title = item.name,
        start = item
            .start_time
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        status = item.status,
        notify_text = match item.notify {
            SessionNotifySettings::Notify =>
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-bell-off-icon lucide-bell-off"><path d="M10.268 21a2 2 0 0 0 3.464 0"/><path d="M17 17H4a1 1 0 0 1-.74-1.673C4.59 13.956 6 12.499 6 8a6 6 0 0 1 .258-1.742"/><path d="m2 2 20 20"/><path d="M8.668 3.01A6 6 0 0 1 18 8c0 2.687.77 4.653 1.707 6.05"/></svg> Turn off Notifications"#,
            SessionNotifySettings::Ignore =>
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-bell-icon lucide-bell"><path d="M10.268 21a2 2 0 0 0 3.464 0"/><path d="M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326"/></svg>Turn on Notifications"#,
        },
        toggle_action = match item.notify {
            SessionNotifySettings::Notify => "ignore_session",
            SessionNotifySettings::Ignore => "notify_session",
        }
    )?;
    Ok(())
}
