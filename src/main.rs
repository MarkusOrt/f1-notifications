//! This is a Discord Bot that notifies a Channel and Group when a new F1 or
//! F1-Feeder session starts.

use std::str::FromStr;

use sentry::{integrations::tracing::EventFilter, types::Dsn};
use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};
use tracing::{error, info, level_filters::LevelFilter};
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{bot::bot_thread, http::http_api};

pub mod bot;
mod error;
mod http;

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
enum Interaction {
    Ping = 1,
    ApplicationCommand,
    MessageComponent,
    Autocomplete,
    ModalSubmit,
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
struct Testing;

#[allow(dead_code)]
#[derive(Debug)]
enum InteractionData {
    Ping,
    ApplicationCommand(Testing),
    MessageComponent(Testing),
    Autocomplete(Testing),
    ModalSubmit(Testing),
}

impl<'de> Deserialize<'de> for InteractionData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let t = value
            .get("kind")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| serde::de::Error::custom("Missing Type"))?;

        let obj = value
            .get("data")
            .ok_or_else(|| serde::de::Error::missing_field("data"))?;
        match t {
            1 => Ok(InteractionData::Ping),
            2 => Ok(Self::ApplicationCommand(
                serde_json::from_value::<Testing>(obj.clone())
                    .map_err(|v| serde::de::Error::custom(format!("{v}")))?,
            )),

            3 => Ok(Self::MessageComponent(
                serde_json::from_value::<Testing>(obj.clone())
                    .map_err(|v| serde::de::Error::custom(format!("{v}")))?,
            )),

            4 => Ok(Self::Autocomplete(
                serde_json::from_value::<Testing>(obj.clone())
                    .map_err(|v| serde::de::Error::custom(format!("{v}")))?,
            )),

            5 => Ok(Self::ModalSubmit(
                serde_json::from_value::<Testing>(obj.to_owned())
                    .map_err(|v| serde::de::Error::custom(format!("{v}")))?,
            )),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown interaction type {t}"
            ))),
        }
    }
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
struct InteractionReceive {
    pub id: String,
    pub application_id: String,
    #[serde(rename = "type")]
    pub kind: Interaction,
    pub token: String,
}

struct RequiredData {
    pub bot_token: String,
    pub public_key: String,
}

impl RequiredData {
    pub fn try_new() -> Result<Self, std::env::VarError> {
        Ok(Self {
            bot_token: std::env::var("BOT_TOKEN")?,
            public_key: std::env::var("PUBLIC_KEY")?,
        })
    }
}

const USER_AGENT: &str = concat!(
    "f1-notifications-client@",
    env!("CARGO_PKG_VERSION"),
    " contact: markus_dev @ discord"
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    _ = dotenvy::dotenv();
    let mut sentry_client = None;
    if let Ok(dsn) = std::env::var("SENTRY_DSN") {
        sentry_client = Some(sentry::init(sentry::ClientOptions {
            release: sentry::release_name!(),
            dsn: Some(Dsn::from_str(&dsn).expect("Valid DSN")),
            sample_rate: 1.0,
            traces_sample_rate: 1.0,
            ..Default::default()
        }));
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(LevelFilter::INFO))
        .with(
            sentry::integrations::tracing::layer().event_filter(|f| match *f.level() {
                tracing::Level::ERROR => EventFilter::Event,
                tracing::Level::INFO => EventFilter::Log | EventFilter::Breadcrumb,
                tracing::Level::WARN => EventFilter::Log | EventFilter::Breadcrumb,
                _ => EventFilter::Ignore,
            }),
        )
        .init();

    info!("App Start up at {}", chrono::Utc::now());

    sentry::start_session();

    let data = match RequiredData::try_new() {
        Ok(d) => d,
        Err(why) => {
            error!("Error gathering required Configuration: {why:#?}");
            return Err(why.into());
        }
    };
    let http = crate::bot::http::Http::new(&data.bot_token);

    if let Err(why) = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel::<()>(1);
            let sd1 = shutdown_rx.resubscribe();

            let db_client = libsql::Builder::new_local("database/db").build().await?;
            let c1 = db_client.connect()?;
            let c2 = db_client.connect()?;

            let http_bot = http.clone();
            let mut js = tokio::task::JoinSet::new();

            js.spawn(async move { bot_thread(sd1, http_bot, c1).await });

            js.spawn(async move { http_api(shutdown_rx, http, data, c2).await });

            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    info!("Shutting Down");
                    shutdown_tx.send(())?;
                }
                res = js.join_next() => {
                    match res {
                        Some(Err(why)) => {
                            info!("Thread stopped due to error (see in sentry). Shutting down.");
                            sentry::capture_error(&why);
                        },
                        Some(Ok(Err(why))) => {
                            info!("Thread stopped due to error (see in sentry). Shutting down.");
                            sentry::capture_error(&why);
                        },
                        _ => ()
                    }

                }
            }
            while let Some(t) = js.join_next().await {
                match t {
                    Err(why) => {
                        sentry::capture_error(&why);
                    }
                    Ok(Err(why)) => {
                        sentry::capture_error(&why);
                    }
                    _ => (),
                }
            }

            Ok::<(), crate::error::Error>(())
        })
    {
        sentry::capture_error(&why);
    }
    sentry::end_session_with_status(sentry::protocol::SessionStatus::Ok);
    if let Some(client) = sentry::Hub::current().client() {
        client.close(Some(std::time::Duration::from_secs(2)));
    }
    drop(sentry_client);
    Ok(())
}
