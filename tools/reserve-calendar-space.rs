use axum::http::HeaderMap;
use f1_bot_types::{MessageKind, Series};
use libsql::params;
use reqwest::header::AUTHORIZATION;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    _ = dotenvy::dotenv();
    let database = libsql::Builder::new_local("./database/db").build().await?;
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bot {}", std::env::var("BOT_TOKEN")?).parse()?,
    );
    let http_client = reqwest::ClientBuilder::new()
        .user_agent(concat!(
            "Calendar-Space-Reserver@",
            env!("CARGO_PKG_VERSION")
        ))
        .default_headers(headers)
        .build()?;

    let conn = database.connect()?;

    let channel = std::env::var("CALENDAR_CHANNEL")?;
    // Try removing all old calendar messages if possible
    {
        let existing_messages = database_messages(&conn).await?;
        for message in existing_messages {
            let response = http_client
                .delete(format!(
                    "https://discord.com/api/v10/channels/{}/messages/{}",
                    channel, message.discord_id
                ))
                .send()
                .await?;
            let status = response.status();
            if !(status.is_success() || status == 404) {
                continue;
            }
            delete_db_message(&conn, message.id).await?;
        }
    }

    // Create new Messages for the Calendar!
    for _ in 0..6 {
        let response = http_client
            .post(format!(
                "https://discord.com/api/v10/channels/{}/messages",
                channel
            ))
            .json(&Content::DEFAULT)
            .send()
            .await?;
        let response = response.error_for_status()?;
        let new_message: CreateMessage = response.json().await?;
        new_db_message(&conn, new_message).await?;
    }

    Ok(())
}

async fn delete_db_message(db_conn: &libsql::Connection, id: u64) -> Result<(), libsql::Error> {
    _ = db_conn
        .execute("DELETE FROM messages WHERE id = ?", params![id])
        .await?;
    Ok(())
}

async fn database_messages(
    db_conn: &libsql::Connection,
) -> Result<Vec<f1_bot_types::Message>, libsql::Error> {
    let mut cursor = db_conn
        .query(
            "SELECT * FROM messages WHERE kind = ?",
            params![MessageKind::Calendar],
        )
        .await?;
    let mut return_value = Vec::with_capacity(6);
    while let Ok(Some(row)) = cursor.next().await {
        return_value.push(libsql::de::from_row(&row).unwrap());
    }

    Ok(return_value)
}

async fn new_db_message(
    db_conn: &libsql::Connection,
    new_message: CreateMessage,
) -> Result<(), libsql::Error> {
    _ = db_conn
        .execute(
            "INSERT INTO messages (discord_id, discord_channel, kind, series, expires_at) VALUES (?, ?, ?, ?, ?)",
            params![
                new_message.id,
                new_message.channel_id,
                MessageKind::Calendar,
                Series::F1,
                "2100-10-10T10:10:10Z"
            ],
        )
        .await?;
    Ok(())
}

#[derive(serde::Serialize)]
struct Content<'a> {
    content: &'a str,
}

impl Content<'static> {
    pub const DEFAULT: Self = Content {
        content: "*Reserved for Calendar*",
    };
}

#[derive(serde::Deserialize)]
struct CreateMessage {
    pub id: String,
    pub channel_id: String,
}
