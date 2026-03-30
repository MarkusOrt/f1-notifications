use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use f1_bot_types::{SessionNotifySettings, SessionStatus};
use libsql::params;
use reqwest::StatusCode;

use crate::{
    error::Error,
    http::{AxumState, auth::User},
};

pub async fn notifications_off(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Path(id): Path<u64>,
) -> Result<StatusCode, crate::error::Error> {
    _ = user;
    let rows = app_state
        .db_pool
        .execute(
            "UPDATE sessions SET notify = ? WHERE id = ?",
            params![SessionNotifySettings::Ignore, id],
        )
        .await?;

    if rows == 0 {
        return Err(Error::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn notifications_on(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Path(id): Path<u64>,
) -> Result<StatusCode, crate::error::Error> {
    _ = user;
    let rows = app_state
        .db_pool
        .execute(
            "UPDATE sessions SET notify = ? WHERE id = ?",
            params![SessionNotifySettings::Notify, id],
        )
        .await?;

    if rows == 0 {
        return Err(Error::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdateSession {
    name: String,
    start_time: DateTime<Utc>,
    status: SessionStatus,
    notify: SessionNotifySettings,
}

pub async fn update_session(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Path(id): Path<u64>,
    Json(update_session): Json<UpdateSession>,
) -> Result<StatusCode, crate::error::Error> {
    _ = user;
    let rows = app_state
        .db_pool
        .execute(
            "UPDATE sessions SET name = ?, start_time = ?, status = ?, notify = ? WHERE id = ?",
            params![
                update_session.name,
                update_session
                    .start_time
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                update_session.status,
                update_session.notify,
                id
            ],
        )
        .await?;

    if rows == 0 {
        return Err(Error::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(serde::Deserialize, Debug)]
pub struct NewSession {
    name: String,
    event_id: u64,
    start_time: DateTime<Utc>,
    status: SessionStatus,
    notify: SessionNotifySettings,
}

pub async fn new_session(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Json(body): Json<NewSession>,
) -> Result<StatusCode, crate::error::Error> {
    _ = user;
    let rows = app_state
        .db_pool
        .execute(
            "INSERT INTO sessions (name, weekend_id, start_time, status, notify, created_at) VALUES(?, ?, ?, ?, ?, ?)",
            params![
                body.name,
                body.event_id,
                body
                    .start_time
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                body.status,
                body.notify,
                Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            ],
        )
        .await.inspect_err(|f| println!("{f}"))?;
    _ = rows;

    Ok(StatusCode::NO_CONTENT)
}
pub async fn delete_session(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Path(id): Path<u64>,
) -> Result<StatusCode, crate::error::Error> {
    _ = user;
    let rows = app_state
        .db_pool
        .execute("DELETE FROM sessions WHERE id = ?", params![id])
        .await?;
    if rows == 0 {
        return Err(Error::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
