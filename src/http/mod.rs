use crate::{Interaction, InteractionReceive, error::ErrResult, http::message::Flags};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Request},
    response::IntoResponse,
    routing::{get, post},
};
use ed25519_dalek::{Signature, VerifyingKey};
use reqwest::{StatusCode, header::CONTENT_TYPE};
use std::fmt::Write;
use tokio::sync::broadcast::Receiver;
use tower::ServiceBuilder;
use tracing::info;

mod message;

#[derive(Clone, Debug)]
struct AxumState<'a> {
    pub public_key: &'a VerifyingKey,
    pub http: crate::bot::http::Http,
}

pub async fn http_api(
    mut shutdown: Receiver<()>,
    http: crate::bot::http::Http,
    data: crate::RequiredData,
    db_conn: libsql::Connection,
) -> ErrResult {
    _ = db_conn;
    let mut public_key = [0u8; 32];
    hex::decode_to_slice(data.public_key, &mut public_key)?;
    let vk = Box::leak(Box::new(VerifyingKey::from_bytes(&public_key)?));

    let router = axum::Router::new()
        .route("/interaction", post(interaction))
        .route("/health", get(health))
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
    sentry::configure_scope(|f| f.set_tag("http.status_code", 404));
    (StatusCode::NOT_FOUND, "Not Found.")
}

async fn health() -> StatusCode {
    sentry::configure_scope(|f| f.set_tag("http.status_code", 200));
    StatusCode::OK
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
    flags: u32,
}

impl InterResponse {
    const TESTING_RESPONSE: Self = Self {
        kind: 4,
        data: Content {
            content: "This is just a test!",
            flags: Flags::EPHEMERAL,
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
        sentry::configure_scope(|f| f.set_tag("http.status_code", 401));
        return (
            StatusCode::UNAUTHORIZED,
            HeaderMap::new(),
            "Unauthorized.".to_owned(),
        );
    };

    let (Ok(signature), Ok(timestamp)) = (signature.to_str(), timestamp.to_str()) else {
        sentry::configure_scope(|f| f.set_tag("http.status_code", 401));
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
    if state
        .public_key
        .verify_strict(message.as_bytes(), &sign)
        .is_err()
    {
        sentry::configure_scope(|f| f.set_tag("http.status_code", 401));
        return (
            StatusCode::UNAUTHORIZED,
            HeaderMap::new(),
            "Unauthorized.".to_owned(),
        );
    }
    let serialized_body: InteractionReceive = serde_json::from_str(&body).unwrap();
    let tx = sentry::configure_scope(|f| f.get_span());

    match serialized_body.kind {
        Interaction::Ping => {
            let response = serde_json::to_string(&DiscordResponse { kind: 1 }).unwrap();
            let mut headers = HeaderMap::new();
            headers.append(CONTENT_TYPE, "application/json".parse().unwrap());

            sentry::configure_scope(|f| f.set_tag("http.status_code", 200));
            (StatusCode::OK, headers, response)
        }
        _ => {
            _ = state
                .http
                .execute_request(
                    &tx,
                    state
                        .http
                        .interaction_response(serialized_body.id, serialized_body.token)
                        .json(&InterResponse::TESTING_RESPONSE),
                )
                .await
                .unwrap();

            sentry::configure_scope(|f| {
                f.set_tag("http.status_code", 202);
                f.get_span()
                    .inspect(|f| f.set_status(sentry::protocol::SpanStatus::Ok));
            });
            (
                StatusCode::ACCEPTED,
                HeaderMap::new(),
                String::with_capacity(0),
            )
        }
    }
}
