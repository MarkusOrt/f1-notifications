use std::fmt::Write;
use std::time::Duration;

use chrono::Utc;
use f1_bot_types::{Session, Weekend};
use tokio::sync::broadcast::Receiver;
use tracing::info;

pub mod calendar;
mod database;
pub mod http;

#[allow(unused)]
const MAX_WEEKENDS_PER_MESSAGE: u32 = 5;

#[derive(serde::Serialize)]
struct CreateMessage<'a> {
    content: &'a str,
}

#[derive(serde::Deserialize, Debug)]
struct CreateMessageResponse {
    id: String,
    channel_id: String,
}

pub async fn bot_thread(
    mut should_shut_down: Receiver<()>,
    http: http::Http,
    db_conn: libsql::Connection,
) {
    info!("Bot thread starting.");
    let mut message_hashes = [0u64; 4];
    loop {
        if let Ok(_) = should_shut_down.try_recv() {
            break;
        }

        let Some(f1_weekend) = database::next_weekend(f1_bot_types::Series::F1, &db_conn)
            .await
            .unwrap()
        else {
            continue;
        };
        let sessions_for_f1 = database::sessions_for_weekend(f1_weekend.id as i32, &db_conn)
            .await
            .unwrap();

        println!("Weekend: {f1_weekend:#?}");
        println!("Sessions: {sessions_for_f1:#?}");
        let msg_content = persistent_msg_f1(&f1_weekend, &sessions_for_f1).unwrap();
        http.create_message("1002285400095719524")
            .json(&CreateMessage {
                content: &msg_content,
            })
            .send()
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_secs(10)).await;
    }
    info!("Bot Thread shutdown");
}

/// Creates the Message String for the F1 Persistent current event message.
/// This one will strike-through any expired sessions.
#[allow(unused)]
pub fn persistent_msg_f1(
    weekend: &f1_bot_types::Weekend,
    sessions: &Vec<Session>,
) -> Result<String, std::fmt::Error> {
    let mut str = String::new();
    writeln!(
        &mut str,
        "## {} {} {}",
        weekend.icon, weekend.year, weekend.name
    )?;
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
#[allow(unused)]
pub fn persistent_msg_feeder(
    data: [Option<(&Weekend, &Vec<Session>)>; 3],
) -> Result<String, std::fmt::Error> {
    let mut str = String::new();
    let mut data_itr = data.into_iter();
    while let Some(Some((weekend, sessions))) = data_itr.next() {
        writeln!(
            &mut str,
            "## {} {} {} {}",
            weekend.icon, weekend.year, weekend.series, weekend.name
        )?;
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
        r#"## testing 2025 testing
> ~~`   testing`: <t:1735718400:F> (<t:1735718400:R>)~~
> `   testing`: <t:7952371200:F> (<t:7952371200:R>)
"#
        .to_string()
    );

    assert_eq!(persistent_msg_feeder([None, None, None]).unwrap(), "");

    let str = persistent_msg_feeder([
        Some((
            &Weekend {
                id: 0,
                name: "testing".to_string(),
                year: 2025,
                start_date: DateTime::parse_from_rfc3339("2025-01-01T06:00:00Z")
                    .unwrap()
                    .into(),
                icon: "testing".to_string(),
                series: f1_bot_types::Series::F2,
                status: f1_bot_types::WeekendStatus::Open,
            },
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
        )),
        None,
        None,
    ])
    .unwrap();
    assert_eq!(
        str,
        r#"## testing 2025 F2 testing
> ~~`   testing`: <t:1735718400:F> (<t:1735718400:R>)~~
"#
    )
}
