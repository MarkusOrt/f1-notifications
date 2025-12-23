use std::fmt::Write;
use std::time::Duration;

use chrono::Utc;
use f1_bot_types::{Session, Weekend};
use tokio::sync::broadcast::Receiver;
use tracing::info;

#[allow(unused)]
const MAX_WEEKENDS_PER_MESSAGE: u32 = 5;

#[allow(unused)]
pub async fn bot_thread(mut should_shut_down: Receiver<()>, http: reqwest::Client) {
    info!("Bot thread starting.");
    loop {
        if let Ok(_) = should_shut_down.try_recv() {
            break;
        }

        tokio::time::sleep(Duration::from_secs(1));
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
        let tz = session.start_date.timestamp();
        let dur = chrono::Duration::seconds(session.duration.into());
        if session.start_date + dur < now {
            writeln!(
                &mut str,
                "> ~~`{0:>10}`: <t:{1}:F> (<t:{1}:R>)~~",
                session.title, tz
            )?;
        } else {
            writeln!(
                &mut str,
                "> `{0:>10}`: <t:{1}:F> (<t:{1}:R>)",
                session.title, tz
            )?;
        }
    }

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
            let tz = session.start_date.timestamp();
            let dur = chrono::Duration::seconds(session.duration.into());
            if session.start_date + dur < now {
                writeln!(
                    &mut str,
                    "> ~~`{0:>10}`: <t:{1}:F> (<t:{1}:R>)~~",
                    session.title, tz
                )?;
            } else {
                writeln!(
                    &mut str,
                    "> `{0:>10}`: <t:{1}:F> (<t:{1}:R>)",
                    session.title, tz
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
            weekend: 0,
            start_date: DateTime::parse_from_rfc3339("2025-01-01T08:00:00Z")
                .unwrap()
                .into(),
            title: "testing".to_string(),
            kind: f1_bot_types::session::SessionKind::Racing,
            duration: 3600,
            notify: f1_bot_types::session::SessionNotifySettings::Notify,
            status: f1_bot_types::session::SessionStatus::Open,
        },
        Session {
            id: 1,
            weekend: 0,
            start_date: DateTime::parse_from_rfc3339("2222-01-01T08:00:00Z")
                .unwrap()
                .into(),
            title: "testing".to_string(),
            kind: f1_bot_types::session::SessionKind::Racing,
            duration: 3600,
            notify: f1_bot_types::session::SessionNotifySettings::Notify,
            status: f1_bot_types::session::SessionStatus::Open,
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
                weekend: 0,
                start_date: DateTime::parse_from_rfc3339("2025-01-01T08:00:00Z")
                    .unwrap()
                    .into(),
                title: "testing".to_string(),
                kind: f1_bot_types::SessionKind::Racing,
                duration: 3600,
                notify: f1_bot_types::SessionNotifySettings::Notify,
                status: f1_bot_types::SessionStatus::Open,
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
