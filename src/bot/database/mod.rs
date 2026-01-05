use f1_bot_types::{Series, Session, Weekend, WeekendStatus};
use libsql::params;

pub async fn weekends_for_series(
    series: Series,
    db_conn: &libsql::Connection,
) -> Result<Vec<Weekend>, libsql::Error> {
    let mut cursor = db_conn
        .query("SELECT * FROM weekends WHERE series = ?", params![series])
        .await?;
    let mut return_value = Vec::with_capacity(10);
    while let Ok(Some(value)) = cursor.next().await {
        return_value.push(libsql::de::from_row::<Weekend>(&value).unwrap());
    }
    Ok(return_value)
}

pub async fn next_weekend(
    series: Series,
    db_conn: &libsql::Connection,
) -> Result<Option<Weekend>, libsql::Error> {
    let mut cursor = db_conn
        .query(
            //"SELECT * FROM weekends WHERE series = ? AND status = ? AND CURRENT_TIMESTAMP > Datetime('start_date', '-7 days') ORDER BY start_date",
            "SELECT * FROM weekends WHERE series = ? AND status = ? ORDER BY start_date",
            params![series, WeekendStatus::Open],
        )
        .await?;
    let val = cursor.next().await?;
    Ok(val.map(|f| libsql::de::from_row(&f).unwrap()))
}

pub async fn sessions_for_weekend(
    weekend_id: i32,
    db_conn: &libsql::Connection,
) -> Result<Vec<Session>, libsql::Error> {
    let mut cursor = db_conn
        .query(
            "SELECT * FROM sessions WHERE weekend_id = ? ORDER BY start_time",
            params![weekend_id],
        )
        .await?;
    let mut return_value = Vec::with_capacity(5);
    while let Ok(Some(row)) = cursor.next().await {
        return_value.push(libsql::de::from_row(&row).unwrap());
    }
    Ok(return_value)
}

pub async fn next_session(
    series: Series,
    db_conn: &libsql::Connection,
) -> Result<Option<Session>, libsql::Error> {
    _ = series;
    _ = db_conn;
    Ok(None)
}
