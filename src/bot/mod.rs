use std::time::Duration;
use std::{fmt::Write, hash::Hash};

use chrono::Utc;
use f1_bot_types::{Message, MessageKind, Series, Session, SessionStatus, Weekend, WeekendStatus};
use libsql::params;
use sentry::protocol::{TraceContext, TraceId};
use tokio::sync::broadcast::Receiver;
use tracing::{info, warn};

use crate::bot::calendar::make_calendar_message_string;
use crate::bot::database::{get_event_message, update_message_hash};
use crate::bot::http::Http;
use crate::error::ErrResult;

pub mod calendar;
mod database;
pub mod http;

#[allow(unused)]
const MAX_WEEKENDS_PER_MESSAGE: u32 = 5;

#[derive(serde::Serialize)]
struct CreateMessage<'a> {
    content: &'a str,
}

#[allow(unused)]
#[derive(serde::Deserialize, Debug)]
struct CreateMessageResponse {
    id: String,
    channel_id: String,
}

pub async fn bot_thread(
    mut should_shut_down: Receiver<()>,
    http: http::Http,
    db_conn: libsql::Connection,
) -> ErrResult {
    info!("Bot thread starting.");
    loop {
        let trace_id = TraceContext::default().trace_id;

        if should_shut_down.try_recv().is_ok() {
            break;
        }

        let f1_channel = std::env::var("F1_CHANNEL")?;
        let feeder_channel = std::env::var("FEEDER_CHANNEL")?;

        'calendar: {
            let calendar_messages =
                database::get_calendar_messages(&db_conn, trace_id, Series::F1).await?;
            if calendar_messages.len() < 5 {
                warn!("Skipping Calendar due to insufficient messages (less than 6)!");
                break 'calendar;
            }
            let weekends = database::weekends_for_series(&db_conn, trace_id, Series::F1).await?;
            let sessions = database::all_sessions(&db_conn, trace_id).await?;

            for (i, message) in calendar_messages.iter().enumerate() {
                let message_string = make_calendar_message_string(&weekends, &sessions, i)?;
                let mut hasher = std::hash::DefaultHasher::new();
                message_string.hash(&mut hasher);
                let new_hash = std::hash::Hasher::finish(&hasher);
                let hash: u64 = message.hash.parse()?;
                if hash == new_hash {
                    continue;
                }

                if message_string.is_empty() {
                    continue;
                }
                let req = http
                    .edit_message(&message.discord_channel, &message.discord_id)
                    .json(&CreateMessage {
                        content: &message_string,
                    });

                let res = http.execute_request(trace_id, req).await?;
                _ = res;

                update_message_hash(&db_conn, trace_id, message.id, new_hash.to_string()).await?;
            }
        }

        'f1_persistent: {
            let Some(f1_weekend) = database::next_weekend(&db_conn, trace_id, Series::F1).await?
            else {
                break 'f1_persistent;
            };
            let sessions_for_f1 =
                database::sessions_for_weekend(&db_conn, trace_id, f1_weekend.id).await?;
            let event_message =
                match database::get_event_message(&db_conn, trace_id, Series::F1).await? {
                    Some(msg) => msg,
                    None => {
                        create_event_message(
                            &db_conn,
                            &http,
                            trace_id,
                            &f1_channel,
                            &f1_weekend,
                            &sessions_for_f1,
                        )
                        .await?
                    }
                };

            if !(sessions_for_f1.is_empty()
                || sessions_for_f1
                    .iter()
                    .any(|f| f.status == SessionStatus::Open))
            {
                let res = http
                    .execute_request(
                        trace_id,
                        http.delete_message(
                            &event_message.discord_channel,
                            &event_message.discord_id,
                        ),
                    )
                    .await?
                    .error_for_status()?;

                _ = res;
                database::delete_message(&db_conn, trace_id, event_message.id).await?;
                database::update(
                    &db_conn,
                    trace_id,
                    "UPDATE weekends SET status = ? WHERE id = ?",
                    params![WeekendStatus::Done, f1_weekend.id],
                )
                .await?;
            }

            let db_hash: u64 = event_message.hash.parse()?;
            let message_content = persistent_msg_f1(&f1_weekend, &sessions_for_f1)?;
            let mut hasher = std::hash::DefaultHasher::new();
            message_content.hash(&mut hasher);
            let hash = std::hash::Hasher::finish(&hasher);
            if hash != db_hash {
                let response = http
                    .execute_request(
                        trace_id,
                        http.edit_message(
                            &event_message.discord_channel,
                            &event_message.discord_id,
                        )
                        .json(&Content {
                            content: &message_content,
                        }),
                    )
                    .await?
                    .error_for_status()?;
                _ = response;
            }
        }

        {
            let next_f2_weekend = database::next_weekend(&db_conn, trace_id, Series::F2).await?;
            let next_f3_weekend = database::next_weekend(&db_conn, trace_id, Series::F3).await?;
            let next_f1a_weekend =
                database::next_weekend(&db_conn, trace_id, Series::F1Academy).await?;
            let f2_sessions = match &next_f2_weekend {
                Some(w) => database::sessions_for_weekend(&db_conn, trace_id, w.id).await?,
                None => Vec::new(),
            };
            let f3_sessions = match &next_f3_weekend {
                Some(w) => database::sessions_for_weekend(&db_conn, trace_id, w.id).await?,
                None => Vec::new(),
            };
            let f1a_sessions = match &next_f1a_weekend {
                Some(w) => database::sessions_for_weekend(&db_conn, trace_id, w.id).await?,
                None => Vec::new(),
            };
            let weekends_message = match get_event_message(&db_conn, trace_id, Series::F2).await? {
                Some(m) => m,
                None => {
                    create_feeder_message(
                        &db_conn,
                        &http,
                        trace_id,
                        &feeder_channel,
                        [
                            (&next_f2_weekend, &f2_sessions),
                            (&next_f3_weekend, &f3_sessions),
                            (&next_f1a_weekend, &f1a_sessions),
                        ],
                    )
                    .await?
                }
            };
            let message_content = persistent_msg_feeder([
                (&next_f2_weekend, &f2_sessions),
                (&next_f3_weekend, &f3_sessions),
                (&next_f1a_weekend, &f1a_sessions),
            ])?;
            let mut hasher = std::hash::DefaultHasher::new();
            message_content.hash(&mut hasher);
            let hash = std::hash::Hasher::finish(&hasher);
            let message_hash: u64 = weekends_message.hash.parse().unwrap_or_default();
            if hash != message_hash || hash == 0 || message_hash == 0 {
                let res = http
                    .execute_request(
                        trace_id,
                        http.edit_message(
                            weekends_message.discord_channel,
                            weekends_message.discord_id,
                        )
                        .json(&Content::new(&message_content)),
                    )
                    .await?
                    .error_for_status()?;
                _ = res;
                database::update_message_hash(
                    &db_conn,
                    trace_id,
                    weekends_message.id,
                    hash.to_string(),
                )
                .await?
            }
        }
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
    info!("Bot Thread shutdown");
    Ok(())
}

/// Creates the Message String for the F1 Persistent current event message.
/// This one will strike-through any expired sessions.
#[allow(unused)]
pub fn persistent_msg_f1(
    weekend: &f1_bot_types::Weekend,
    sessions: &Vec<Session>,
) -> Result<String, std::fmt::Error> {
    let mut str = String::new();
    writeln!(&mut str, "## {} {}", weekend.icon, weekend.name)?;
    let now = Utc::now();
    for session in sessions {
        let tz = session.start_time.timestamp();
        let dur = chrono::Duration::seconds(session.duration.into());
        if session.start_time + dur < now {
            writeln!(
                &mut str,
                "> ~~`{0:>10}`: <t:{1}:F> (<t:{1}:R>)~~",
                session.name, tz
            )?;
        } else {
            writeln!(
                &mut str,
                "> `{0:>10}`: <t:{1}:F> (<t:{1}:R>)",
                session.name, tz
            )?;
        }
    }

    writeln!(
        &mut str,
        "\nUse <id:customize> and get the `f1-notifications` role to receive notifications!"
    );

    Ok(str)
}

/// Creates the Persistent next-event message for Feeder Series (F2, F3, F1A)
/// this one will be a single message for all three series.
pub fn persistent_msg_feeder(
    data: [(&Option<Weekend>, &Vec<Session>); 3],
) -> Result<String, std::fmt::Error> {
    let mut str = String::new();
    writeln!(&mut str, "# Next Feeder Events")?;
    let mut data_itr = data.into_iter();
    while let Some((Some(weekend), sessions)) = data_itr.next() {
        writeln!(
            &mut str,
            "## {} {} {}",
            weekend.icon, weekend.series, weekend.name
        )?;
        if sessions.is_empty() {
            writeln!(&mut str, "> :hourglass: Times are TBC")?;
        }
        let now = Utc::now();
        for session in sessions {
            let tz = session.start_time.timestamp();
            let dur = chrono::Duration::seconds(session.duration.into());
            if session.start_time + dur < now {
                writeln!(
                    &mut str,
                    "> ~~`{0:>10}`: <t:{1}:F> (<t:{1}:R>)~~",
                    session.name, tz
                )?;
            } else {
                writeln!(
                    &mut str,
                    "> `{0:>10}`: <t:{1}:F> (<t:{1}:R>)",
                    session.name, tz
                )?;
            }
        }
    }
    writeln!(
        &mut str,
        "\nGo to <id:customize> and select the [Series]-Notifications role to receive notifications for each series.\nAll Times are in your timezone."
    )?;

    Ok(str)
}

/// Tests whether or not the message generation works.
/// Sessions that are done should be shown strike-through (~~)
#[test]
pub fn test_message() {
    use chrono::DateTime;
    let weekend = f1_bot_types::Weekend {
        id: 0,
        name: "testing".to_string(),
        year: 2025,
        start_date: DateTime::parse_from_rfc3339("2025-01-01T06:00:00Z")
            .unwrap()
            .into(),
        icon: "testing".to_string(),
        series: f1_bot_types::Series::F1,
        status: f1_bot_types::WeekendStatus::Open,
    };
    let sessions = vec![
        Session {
            id: 0,
            weekend_id: 0,
            start_time: DateTime::parse_from_rfc3339("2025-01-01T08:00:00Z")
                .unwrap()
                .into(),
            name: "testing".to_string(),
            duration: 3600,
            notify: f1_bot_types::session::SessionNotifySettings::Notify,
            status: f1_bot_types::session::SessionStatus::Open,
            created_at: std::time::UNIX_EPOCH.into(),
        },
        Session {
            id: 1,
            weekend_id: 0,
            start_time: DateTime::parse_from_rfc3339("2222-01-01T08:00:00Z")
                .unwrap()
                .into(),
            name: "testing".to_string(),
            duration: 3600,
            notify: f1_bot_types::session::SessionNotifySettings::Notify,
            status: f1_bot_types::session::SessionStatus::Open,
            created_at: std::time::UNIX_EPOCH.into(),
        },
    ];

    let str = persistent_msg_f1(&weekend, &sessions).unwrap();

    assert_eq!(
        str,
        r#"## testing testing
> ~~`   testing`: <t:1735718400:F> (<t:1735718400:R>)~~
> `   testing`: <t:7952371200:F> (<t:7952371200:R>)

Use <id:customize> and get the `f1-notifications` role to receive notifications!
"#
        .to_string()
    );

    assert_eq!(
        persistent_msg_feeder([
            (&None, &Vec::new()),
            (&None, &Vec::new()),
            (&None, &Vec::new())
        ])
        .unwrap(),
        r#"# Next Feeder Events

Go to <id:customize> and select the [Series]-Notifications role to receive notifications for each series.
All Times are in your timezone.
"#
    );

    let str = persistent_msg_feeder([
        (
            &Some(Weekend {
                id: 0,
                name: "testing".to_string(),
                year: 2025,
                start_date: DateTime::parse_from_rfc3339("2025-01-01T06:00:00Z")
                    .unwrap()
                    .into(),
                icon: "testing".to_string(),
                series: f1_bot_types::Series::F2,
                status: f1_bot_types::WeekendStatus::Open,
            }),
            &vec![Session {
                id: 0,
                weekend_id: 0,
                start_time: DateTime::parse_from_rfc3339("2025-01-01T08:00:00Z")
                    .unwrap()
                    .into(),
                name: "testing".to_string(),
                duration: 3600,
                notify: f1_bot_types::SessionNotifySettings::Notify,
                status: f1_bot_types::SessionStatus::Open,
                created_at: std::time::UNIX_EPOCH.into(),
            }],
        ),
        (&None, &Vec::new()),
        (&None, &Vec::new()),
    ])
    .unwrap();
    assert_eq!(
        str,
        r#"# Next Feeder Events
## testing F2 testing
> ~~`   testing`: <t:1735718400:F> (<t:1735718400:R>)~~

Go to <id:customize> and select the [Series]-Notifications role to receive notifications for each series.
All Times are in your timezone.
"#
    )
}

#[derive(serde::Serialize)]
struct Content<'a> {
    content: &'a str,
}

impl<'a> Content<'a> {
    pub fn new(str: &'a str) -> Self {
        Self { content: str }
    }
}

async fn create_event_message(
    db_conn: &libsql::Connection,
    http: &Http,
    trace_id: TraceId,
    channel: &str,
    weekend: &f1_bot_types::Weekend,
    sessions: &Vec<Session>,
) -> ErrResult<Message> {
    let content = persistent_msg_f1(weekend, sessions)?;
    let response = http
        .execute_request(
            trace_id,
            http.create_message(channel)
                .json(&Content { content: &content }),
        )
        .await?
        .error_for_status()?;

    let mut hasher = std::hash::DefaultHasher::new();
    content.hash(&mut hasher);
    let new_hash = std::hash::Hasher::finish(&hasher).to_string();

    let new_message_data: CreateMessageResponse = response.json().await?;
    _ = db_conn.execute(
        "INSERT INTO messages (discord_id, discord_channel, kind, series, expires_at, hash) VALUES (?, ?, ?, ?, ?, ?)", 
        params![
            new_message_data.id.clone(),
            new_message_data.channel_id.clone(),
            MessageKind::Weekend,
            Series::F1,
            (Utc::now() + chrono::Duration::days(100)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            new_hash.clone()
        ]).await?;
    let id = db_conn.last_insert_rowid();

    let message = Message {
        id: id as u64,
        discord_id: new_message_data.id,
        discord_channel: new_message_data.channel_id,
        kind: f1_bot_types::MessageKind::Weekend,
        series: Series::F1,
        expires_at: Utc::now() + chrono::Duration::days(100),
        created_at: Utc::now(),
        hash: new_hash,
    };

    Ok(message)
}

async fn create_feeder_message(
    db_conn: &libsql::Connection,
    http: &Http,
    trace_id: TraceId,
    channel: &str,
    data: [(&Option<Weekend>, &Vec<Session>); 3],
) -> ErrResult<Message> {
    let content = persistent_msg_feeder(data)?;
    let mut hasher = std::hash::DefaultHasher::new();
    content.hash(&mut hasher);
    let hash = std::hash::Hasher::finish(&hasher);
    let expires_at = Utc::now() + chrono::Duration::days(100);
    let res = http
        .execute_request(
            trace_id,
            http.create_message(channel).json(&Content::new(&content)),
        )
        .await?
        .error_for_status()?;
    let new_message_data: CreateMessageResponse = res.json().await?;
    let new_id = database::insert(
        db_conn,
        trace_id,
        r#"INSERT INTO messages
    (discord_channel, discord_id, kind, series, hash, expires_at) 
    VALUES (?, ?, ?, ?, ?, ?)"#,
        params![
            new_message_data.channel_id.clone(),
            new_message_data.id.clone(),
            MessageKind::Weekend,
            Series::F2,
            hash.to_string(),
            expires_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        ],
    )
    .await?;

    Ok(Message {
        id: new_id as u64,
        discord_channel: new_message_data.channel_id,
        discord_id: new_message_data.id,
        kind: MessageKind::Weekend,
        series: Series::F2,
        hash: hash.to_string(),
        expires_at,
        created_at: Utc::now(),
    })
}
