use axum::{
    Json,
    extract::{Path, State},
    response::Html,
};
use chrono::{DateTime, Utc};
use f1_bot_types::Series;
use libsql::params::IntoValue;
use reqwest::StatusCode;

use crate::{
    bot::database,
    error::{Error, FormValidation},
    http::{AxumState, auth::User, html::HtmlBuilder},
};

pub async fn event_dialog(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Path(event_id): Path<u64>,
) -> Result<Html<String>, crate::error::Error> {
    _ = user;
    let weekend = database::weekend(&app_state.db_pool, event_id).await?;
    let html = HtmlBuilder::with_capacity(2048);

    Ok(html.edit_event(&weekend)?.into())
}

pub async fn delete_event(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Path(event_id): Path<u64>,
) -> Result<StatusCode, crate::error::Error> {
    _ = user;
    let rows = app_state
        .db_pool
        .execute(
            "DELETE FROM weekends WHERE id = ?",
            libsql::params![event_id],
        )
        .await
        .inspect_err(|f| println!("{f}"))?;
    if rows == 0 {
        Ok(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

#[derive(serde::Deserialize)]
pub enum EventStatus {
    Open,
    Done,
}

#[derive(serde::Deserialize)]
pub struct UpdateEvent {
    name: String,
    icon: String,
    start_date: DateTime<Utc>,
    series: Series,
    status: EventStatus,
}

impl IntoValue for EventStatus {
    fn into_value(self) -> libsql::Result<libsql::Value> {
        Ok(match self {
            EventStatus::Open => libsql::Value::Text("Open".to_owned()),
            EventStatus::Done => libsql::Value::Text("Done".to_owned()),
        })
    }
}

pub async fn update_event(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Path(event_id): Path<u64>,
    Json(body): Json<UpdateEvent>,
) -> Result<StatusCode, crate::error::Error> {
    _ = user;
    let mut validation = FormValidation::new();
    if !body.icon.is_ascii() || body.icon.len() != 2 {
        validation.0.insert(
            "icon".to_owned(),
            Some("Icon needs to be ASCII only and 2 characters.".to_owned()),
        );
        return Err(Error::Form(validation));
    }
    if update_db_event(&app_state.db_pool, event_id, body)
        .await
        .inspect_err(|f| println!("{f}"))?
        == 0
    {
        return Err(crate::error::Error::NotFound);
    }
    Ok(StatusCode::OK)
}

#[derive(serde::Deserialize)]
pub struct NewWeekend {
    name: String,
    icon: String,
    start_date: DateTime<Utc>,
    series: Series,
    status: EventStatus,
}

pub async fn new_weekend(
    user: User,
    State(app_state): State<AxumState<'_>>,
    Json(new_weekend): Json<NewWeekend>,
) -> Result<StatusCode, crate::error::Error> {
    let mut validation = FormValidation::new();
    _ = user;
    if !new_weekend.icon.is_ascii() {
        validation.0.insert(
            "icon".to_owned(),
            Some("Icon needs to be ASCII only".to_owned()),
        );
        return Err(Error::Form(validation));
    }
    create_db_event(&app_state.db_pool, new_weekend).await?;
    Ok(StatusCode::CREATED)
}

async fn create_db_event(db: &libsql::Connection, data: NewWeekend) -> Result<u64, libsql::Error> {
    let created_at = Utc::now();
    db.execute("INSERT INTO weekends (name, icon, start_date, series, status, created_at) VALUES (?, ?, ?, ?, ?, ?)", libsql::params![
        data.name,
        format!(":flag_{}:", data.icon),
        data.start_date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        data.series,
        data.status,
        created_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    ]).await
}

async fn update_db_event(
    db: &libsql::Connection,
    id: u64,
    data: UpdateEvent,
) -> Result<u64, libsql::Error> {
    let start_date = data
        .start_date
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let icon = format!(":flag_{}:", data.icon);
    db.execute(
        "UPDATE weekends SET name = ?, icon = ?, start_date = ?, series = ?, status = ? WHERE id = ?",
        libsql::params![
            data.name,
            icon,
            start_date,
            data.series,
            data.status,
            id
        ],
    )
    .await
}
