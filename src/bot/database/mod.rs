#![allow(unused)]
use f1_bot_types::{Message, MessageKind, Series, Session, Weekend, WeekendStatus};
use libsql::params;
use sentry::{TransactionContext, protocol::TraceId};
use serde::de::DeserializeOwned;

use crate::error::ErrResult;

pub async fn weekends_for_series(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    series: Series,
) -> ErrResult<Vec<Weekend>> {
    self::fetch(
        db_conn,
        trace_id,
        r#"SELECT * FROM weekends 
        WHERE series = ? ORDER BY start_date ASC"#,
        params![series],
    )
    .await
}

pub async fn fetch<T: DeserializeOwned + Sized>(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> ErrResult<Vec<T>> {
    let tx = sentry::start_transaction(TransactionContext::new_with_trace_id(sql, "db", trace_id));
    tx.set_tag("db.operation", "SELECT");
    tx.set_extra("db.statement", sql.into());

    let mut cursor = db_conn.query(sql, params).await?;
    let mut return_value: Vec<T> = Vec::new();
    while let Ok(Some(row)) = cursor.next().await {
        return_value.push(libsql::de::from_row(&row)?);
    }
    tx.set_data("rows_returned", return_value.len().into());
    tx.set_status(sentry::protocol::SpanStatus::Ok);
    tx.finish();
    Ok(return_value)
}

pub async fn fetch_optional<T: DeserializeOwned + Sized>(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> ErrResult<Option<T>> {
    let tx = sentry::start_transaction(TransactionContext::new_with_trace_id(sql, "db", trace_id));
    tx.set_tag("db.operation", "SELECT");
    tx.set_extra("db.statement", sql.into());

    let mut cursor = db_conn.query(sql, params).await?;
    let mut return_value: Vec<T> = Vec::new();
    if let Ok(Some(row)) = cursor.next().await {
        tx.set_data("rows_returned", 1.into());
        tx.set_status(sentry::protocol::SpanStatus::Ok);
        tx.finish();
        Ok(Some(libsql::de::from_row(&row)?))
    } else {
        tx.set_data("rows_returned", 0.into());
        tx.set_status(sentry::protocol::SpanStatus::Ok);
        tx.finish();
        Ok(None)
    }
}

pub async fn update(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> Result<(), libsql::Error> {
    let tx = sentry::start_transaction(TransactionContext::new_with_trace_id(sql, "db", trace_id));
    tx.set_tag("db.operation", "UPDATE");
    tx.set_extra("db.statement", sql.into());
    let res = db_conn.execute(sql, params).await?;
    tx.set_tag("rows_affected", res);
    tx.set_status(sentry::protocol::SpanStatus::Ok);
    tx.finish();
    Ok(())
}

pub async fn next_weekend(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    series: Series,
) -> ErrResult<Option<Weekend>> {
    self::fetch_optional(
        db_conn,
        trace_id,
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
    trace_id: TraceId,
    weekend_id: u64,
) -> ErrResult<Vec<Session>> {
    self::fetch(
        db_conn,
        trace_id,
        r#"SELECT * FROM sessions 
            WHERE weekend_id = ? 
            ORDER BY start_time"#,
        params![weekend_id],
    )
    .await
}

pub async fn next_session(
    series: Series,
    db_conn: &libsql::Connection,
) -> Result<Option<Session>, libsql::Error> {
    _ = series;
    _ = db_conn;
    Ok(None)
}

pub async fn update_message_hash(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    message_id: u64,
    new_hash: String,
) -> Result<(), libsql::Error> {
    self::update(
        db_conn,
        trace_id,
        "UPDATE messages SET hash = ? WHERE id = ?",
        params![new_hash, message_id],
    )
    .await
}

pub async fn get_calendar_messages(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    series: Series,
) -> ErrResult<Vec<Message>> {
    self::fetch(
        db_conn,
        trace_id,
        r#"SELECT * FROM messages 
            WHERE series = ? 
            AND kind = ?"#,
        params![series, MessageKind::Calendar],
    )
    .await
}

pub async fn all_sessions(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
) -> ErrResult<Vec<Session>> {
    self::fetch(
        db_conn,
        trace_id,
        "SELECT * FROM sessions ORDER BY start_time ASC",
        params![],
    )
    .await
}

pub async fn insert(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> Result<i64, libsql::Error> {
    let tx = sentry::start_transaction(TransactionContext::new_with_trace_id(sql, "db", trace_id));
    tx.set_tag("db.operation", "INSERT");
    tx.set_extra("db.statement", sql.into());
    let res = db_conn.execute(sql, params).await?;
    tx.set_tag("rows_affected", res);
    tx.set_status(sentry::protocol::SpanStatus::Ok);
    tx.finish();
    Ok(db_conn.last_insert_rowid())
}

pub async fn get_event_message(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    series: Series,
) -> ErrResult<Option<Message>> {
    fetch_optional(
        db_conn,
        trace_id,
        "SELECT * FROM messages WHERE series = ? AND kind = ? LIMIT 1",
        params![series, MessageKind::Weekend],
    )
    .await
}

pub async fn delete(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    sql: &str,
    params: impl libsql::params::IntoParams,
) -> Result<(), libsql::Error> {
    let tx = sentry::start_transaction(TransactionContext::new_with_trace_id(sql, "db", trace_id));
    tx.set_tag("db.operation", "DELETE");
    tx.set_extra("db.statement", sql.into());
    let res = db_conn.execute(sql, params).await?;
    tx.set_tag("rows_affected", res);
    tx.set_status(sentry::protocol::SpanStatus::Ok);
    tx.finish();
    Ok(())
}

pub async fn delete_message(
    db_conn: &libsql::Connection,
    trace_id: TraceId,
    message_id: u64,
) -> Result<(), libsql::Error> {
    self::delete(
        db_conn,
        trace_id,
        "DELETE FROM messages WHERE id = ?",
        params![message_id],
    )
    .await
}
