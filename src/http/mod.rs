use crate::{Interaction, InteractionReceive};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request},
    response::IntoResponse,
    routing::post,
};
use ed25519_dalek::{Signature, VerifyingKey};
use reqwest::{StatusCode, header::CONTENT_TYPE};
use std::fmt::Write;
use tokio::sync::broadcast::Receiver;
use tower::ServiceBuilder;
use tracing::info;

#[derive(Clone, Debug)]
struct AxumState<'a> {
    pub public_key: &'a VerifyingKey,
    pub http: reqwest::Client,
}

pub async fn http_api(
    mut shutdown: Receiver<()>,
    http: reqwest::Client,
    data: crate::RequiredData,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut public_key = [0u8; 32];
    hex::decode_to_slice(data.public_key, &mut public_key)?;
    let vk = Box::leak(Box::new(VerifyingKey::from_bytes(&public_key)?));

    let router = axum::Router::new()
        .route("/interaction", post(interaction))
        .with_state(AxumState {
            public_key: vk,
            http,
        })
        .fallback(fallback)
        .layer(
            ServiceBuilder::new()
                .layer(sentry::integrations::tower::NewSentryLayer::<Request<Body>>::new_from_top())
                .layer(sentry::integrations::tower::SentryHttpLayer::new().enable_transaction()),
        )
        .into_make_service();

    let tcp_listener = tokio::net::TcpListener::bind("0.0.0.0:8123").await?;
    info!("HTTP API bound to {}", tcp_listener.local_addr()?);
    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async move {
            shutdown.recv().await.unwrap();
        })
        .await?;
    Ok(())
}

async fn fallback() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not Found.")
}

#[derive(serde::Serialize)]
struct DiscordResponse {
    #[serde(rename = "type")]
    kind: u32,
}

#[derive(serde::Serialize)]
struct InterResponse {
    #[serde(rename = "type")]
    kind: u8,
    data: Content,
}

#[derive(serde::Serialize)]
struct Content {
    content: &'static str,
    flags: u8,
}

impl InterResponse {
    const TESTING_RESPONSE: Self = Self {
        kind: 4,
        data: Content {
            content: "This is just a test!",
            flags: 1 << 6,
        },
    };
}

async fn interaction(
    State(state): State<AxumState<'_>>,
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    let (Some(signature), Some(timestamp)) = (
        headers.get("X-Signature-Ed25519"),
        headers.get("X-Signature-Timestamp"),
    ) else {
        return (
            StatusCode::UNAUTHORIZED,
            HeaderMap::new(),
            "Unauthorized.".to_owned(),
        );
    };

    let (Ok(signature), Ok(timestamp)) = (signature.to_str(), timestamp.to_str()) else {
        return (
            StatusCode::UNAUTHORIZED,
            HeaderMap::new(),
            "Unauthorized.".to_owned(),
        );
    };

    let mut decoded_signature = [0u8; 64];
    hex::decode_to_slice(signature, &mut decoded_signature).unwrap();
    let sign = Signature::from_bytes(&decoded_signature);
    let mut message = String::with_capacity(timestamp.len() + body.len());
    message.write_str(timestamp).unwrap();
    message.write_str(&body).unwrap();
    if let Err(why) = state.public_key.verify_strict(message.as_bytes(), &sign) {
        info!("{why}");
        return (
            StatusCode::UNAUTHORIZED,
            HeaderMap::new(),
            "Unauthorized.".to_owned(),
        );
    }

    let serialized_body: InteractionReceive = serde_json::from_str(&body).unwrap();
    println!("{serialized_body:#?}");
    match serialized_body.kind {
        Interaction::Ping => {
            let response = serde_json::to_string(&DiscordResponse { kind: 1 }).unwrap();
            let mut headers = HeaderMap::new();
            headers.append(CONTENT_TYPE, "application/json".parse().unwrap());

            (StatusCode::OK, headers, response)
        }
        _ => {
            _ = state
                .http
                .post(format!(
                    "https://discord.com/api/v10/interactions/{}/{}/callback",
                    serialized_body.id, serialized_body.token
                ))
                .json(&InterResponse::TESTING_RESPONSE)
                .send()
                .await
                .unwrap();
            (
                StatusCode::ACCEPTED,
                HeaderMap::new(),
                String::with_capacity(0),
            )
        }
    }
}
