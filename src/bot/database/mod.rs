#![allow(unused)]
use chrono::{DateTime, Datelike, Utc};
use f1_bot_types::{Message, MessageKind, Series, Session, SessionStatus, Weekend, WeekendStatus};
use libsql::{de::from_row, ffi::SQLITE_IOCAP_ATOMIC8K, params};
use serde::de::DeserializeOwned;

use crate::error::ErrResult;

pub async fn weekends_for_series(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    series: Series,
) -> ErrResult<Vec<Weekend>> {
    self::fetch(
        db_conn,
        tx,
        r#"SELECT * FROM weekends 
        WHERE series = ? ORDER BY start_date ASC"#,
        params![series],
    )
    .await
}

pub async fn session(db_conn: &libsql::Connection, session_id: u64) -> ErrResult<Option<Session>> {
    let mut cursor = db_conn
        .query("SELECT * FROM sessions WHERE id = ?", params![session_id])
        .await?;
    Ok(match cursor.next().await? {
        Some(r) => Some(libsql::de::from_row::<Session>(&r)?),
        None => None,
    })
}

pub async fn fetch<T: DeserializeOwned + Sized>(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> ErrResult<Vec<T>> {
    let span = tx.start_child("db", sql);
    span.set_tag("db.operation", "SELECT");
    span.set_data("db.statement", sql.into());

    let mut cursor = db_conn.query(sql, params).await?;
    let mut return_value: Vec<T> = Vec::new();
    while let Ok(Some(row)) = cursor.next().await {
        return_value.push(libsql::de::from_row(&row)?);
    }
    span.set_data("rows_returned", return_value.len().into());
    span.set_status(sentry::protocol::SpanStatus::Ok);
    span.finish();
    Ok(return_value)
}

pub async fn fetch_optional<T: DeserializeOwned + Sized>(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> ErrResult<Option<T>> {
    let span = tx.start_child("db", sql);
    span.set_tag("db.operation", "SELECT");
    span.set_data("db.statement", sql.into());

    let mut cursor = db_conn.query(sql, params).await?;
    let mut return_value: Vec<T> = Vec::new();
    if let Ok(Some(row)) = cursor.next().await {
        span.set_data("rows_returned", 1.into());
        span.set_status(sentry::protocol::SpanStatus::Ok);
        span.finish();
        Ok(Some(libsql::de::from_row(&row)?))
    } else {
        span.set_data("rows_returned", 0.into());
        span.set_status(sentry::protocol::SpanStatus::Ok);
        span.finish();
        Ok(None)
    }
}

pub async fn update(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> ErrResult {
    let span = tx.start_child("db", sql);
    span.set_tag("db.operation", "UPDATE");
    span.set_data("db.statement", sql.into());
    let res = db_conn.execute(sql, params).await?;
    span.set_tag("rows_affected", res);
    span.set_status(sentry::protocol::SpanStatus::Ok);
    span.finish();
    Ok(())
}

pub async fn next_weekend(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    series: Series,
) -> ErrResult<Option<Weekend>> {
    self::fetch_optional(
        db_conn,
        tx,
        r#"SELECT * FROM weekends 
            WHERE series = ? 
            AND status = ? 
            ORDER BY start_date
            LIMIT 1"#,
        params![series, WeekendStatus::Open],
    )
    .await
}

pub async fn sessions_for_weekend(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    weekend_id: u64,
) -> ErrResult<Vec<Session>> {
    self::fetch(
        db_conn,
        tx,
        r#"SELECT * FROM sessions 
            WHERE weekend_id = ? 
            ORDER BY start_time"#,
        params![weekend_id],
    )
    .await
}

pub async fn sessions_for_weekend_notx(
    db_conn: &libsql::Connection,
    weekend_id: u64,
) -> ErrResult<Vec<Session>> {
    let mut cursor = db_conn
        .query(
            "SELECT * FROM sessions WHERE weekend_id = ? ORDER BY start_time",
            params![weekend_id],
        )
        .await?;
    let mut return_value = Vec::with_capacity(5);

    while let Ok(Some(row)) = cursor.next().await {
        return_value.push(libsql::de::from_row(&row)?);
    }
    Ok(return_value)
}

pub async fn weekend(db_conn: &libsql::Connection, weekend_id: u64) -> ErrResult<Weekend> {
    let mut cursor = db_conn
        .query("SELECT * FROM weekends WHERE id = ?", params![weekend_id])
        .await?;
    if let Some(row) = cursor.next().await? {
        return Ok(libsql::de::from_row(&row)?);
    } else {
        return Err(crate::error::Error::NotFound);
    }
}

pub async fn next_session(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    weekend_id: u64,
) -> ErrResult<Option<Session>> {
    self::fetch_optional(
        db_conn,
        tx,
        r#"SELECT * FROM sessions 
        WHERE status != ? 
        AND weekend_id = ? 
        ORDER BY start_time ASC
        LIMIT 1"#,
        params![SessionStatus::Finished, weekend_id],
    )
    .await
}

pub async fn update_message_hash(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    message_id: u64,
    new_hash: String,
) -> ErrResult {
    self::update(
        db_conn,
        tx,
        "UPDATE messages SET hash = ? WHERE id = ?",
        params![new_hash, message_id],
    )
    .await
}

pub async fn get_calendar_messages(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    series: Series,
) -> ErrResult<Vec<Message>> {
    self::fetch(
        db_conn,
        tx,
        r#"SELECT * FROM messages 
            WHERE series = ? 
            AND kind = ?"#,
        params![series, MessageKind::Calendar],
    )
    .await
}

pub async fn all_sessions(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
) -> ErrResult<Vec<Session>> {
    self::fetch(
        db_conn,
        tx,
        "SELECT * FROM sessions ORDER BY start_time ASC",
        params![],
    )
    .await
}

pub async fn insert(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> ErrResult<i64> {
    let span = tx.start_child("db", sql);
    span.set_tag("db.operation", "INSERT");
    span.set_data("db.statement", sql.into());
    let res = db_conn.execute(sql, params).await?;
    span.set_tag("rows_affected", res);
    span.set_status(sentry::protocol::SpanStatus::Ok);
    span.finish();
    Ok(db_conn.last_insert_rowid())
}

pub async fn get_event_message(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    series: Series,
) -> ErrResult<Option<Message>> {
    fetch_optional(
        db_conn,
        tx,
        "SELECT * FROM messages WHERE series = ? AND kind = ? LIMIT 1",
        params![series, MessageKind::Weekend],
    )
    .await
}

pub async fn delete(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> Result<(), libsql::Error> {
    let span = tx.start_child("db", sql);
    span.set_tag("db.operation", "DELETE");
    span.set_data("db.statement", sql.into());
    let res = db_conn.execute(sql, params).await?;
    span.set_tag("rows_affected", res);
    span.set_status(sentry::protocol::SpanStatus::Ok);
    span.finish();
    Ok(())
}

pub async fn all_weekends(
    db_conn: &libsql::Connection,
) -> Result<Vec<Weekend>, crate::error::Error> {
    let mut cursor = db_conn
        .query("SELECT * FROM weekends ORDER BY start_date", params![])
        .await?;
    let mut return_value = Vec::with_capacity(24);
    while let Ok(Some(row)) = cursor.next().await {
        return_value.push(libsql::de::from_row(&row)?);
    }
    Ok(return_value)
}

pub async fn delete_message(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    message_id: u64,
) -> Result<(), libsql::Error> {
    self::delete(
        db_conn,
        tx,
        "DELETE FROM messages WHERE id = ?",
        params![message_id],
    )
    .await
}

pub async fn mark_session_finished(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    session_id: i64,
) -> ErrResult {
    self::update(
        db_conn,
        tx,
        "UPDATE sessions SET status = ? WHERE id = ?",
        params![SessionStatus::Finished, session_id],
    )
    .await
}

pub async fn expired_messages(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
) -> ErrResult<Vec<Message>> {
    self::fetch(
        db_conn,
        tx,
        "SELECT * FROM messages WHERE strftime('%Y-%m-%dT%H:%M:%SZ', CURRENT_TIMESTAMP) > expires_at",
        (),
    )
    .await
}

pub async fn new_notify_message(
    db_conn: &libsql::Connection,
    tx: &sentry::Transaction,
    channel: String,
    discord_id: String,
    expiry: DateTime<Utc>,
    series: Series,
) -> ErrResult<i64> {
    self::insert(
        db_conn,
        tx,
        r#"INSERT INTO messages
    (discord_channel, discord_id, expires_at, kind, series) VALUES (?, ?, ?, ?, ?)"#,
        params![
            channel,
            discord_id,
            expiry.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            MessageKind::Notification,
            series,
        ],
    )
    .await
}
