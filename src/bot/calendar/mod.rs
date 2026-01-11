use std::fmt::Write;

use chrono::{Datelike, Utc};
use f1_bot_types::{Session, Weekend};

pub fn make_calendar_message_string(
    weekends: &[Weekend],
    sessions: &[Session],
    num: usize,
) -> Result<String, std::fmt::Error> {
    let chunk_size = 5;
    let start = chunk_size * num;
    if start > weekends.len() {
        return Ok(String::from("-# reserved message"));
    }
    let end = (start + chunk_size).min(weekends.len());

    let mut return_value = String::with_capacity(512);
    if num == 0 {
        writeln!(&mut return_value, "# F1 {} Calendar\n", Utc::now().year())?;
    }

    for weekend in weekends[start..end].iter() {
        let mut this_sessions = Vec::with_capacity(5);
        for session in sessions
            .iter()
            .filter(|f| f.weekend_id == weekend.id as i64)
        {
            this_sessions.push(session);
        }

        message_calendar(weekend, &this_sessions, &mut return_value)?;
    }

    Ok(return_value)
}

pub fn message_calendar(
    weekend: &f1_bot_types::Weekend,
    sessions: &Vec<&Session>,
    str: &mut String,
) -> Result<(), std::fmt::Error> {
    writeln!(str, "## {} {}", weekend.icon, weekend.name)?;
    let now = Utc::now();
    for session in sessions {
        let tz = session.start_time.timestamp();
        let dur = chrono::Duration::seconds(session.duration.into());
        if session.start_time + dur < now {
            writeln!(
                str,
                "> ~~`{0:>10}`: <t:{1}:F> (<t:{1}:R>)~~",
                session.name, tz
            )?;
        } else {
            writeln!(str, "> `{0:>10}`: <t:{1}:F> (<t:{1}:R>)", session.name, tz)?;
        }
    }
    Ok(())
}
