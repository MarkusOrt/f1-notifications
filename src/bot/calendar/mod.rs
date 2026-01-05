use f1_bot_types::{Message, Series};
use libsql::params;



pub async fn get_calendar_messages(db_conn: &libsql::Connection, series: Series) -> Result<Vec<Message>, libsql::Error> {
    let mut cursor = db_conn.query("SELECT * FROM messages WHERE series = ?", params![series.to_str()]).await?;

    Ok(vec![])
}
