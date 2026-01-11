#![allow(dead_code)]
//! HTTP Client Handler

use std::{
    fmt::Display,
    time::Duration,
};

use axum::http::HeaderMap;
use reqwest::{RequestBuilder, Response, header::AUTHORIZATION};
use sentry::{
    TransactionContext,
    protocol::{SpanStatus, TraceId},
};
use tracing::info;

use crate::USER_AGENT;

#[derive(Clone, Debug)]
pub struct Http(reqwest::Client);

impl Http {
    const BASE_URL: &'static str = "https://discord.com/api/v10";

    pub fn new(bot_token: impl Display) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, format!("Bot {bot_token}").parse().unwrap());

        Self(
            reqwest::ClientBuilder::new()
                .user_agent(USER_AGENT)
                .default_headers(headers)
                .build()
                .unwrap(),
        )
    }

    pub fn create_message(&self, channel_id: impl Display) -> RequestBuilder {
        self.0.post(format!(
            "{}/channels/{}/messages",
            Self::BASE_URL,
            channel_id
        ))
    }

    pub fn edit_message(
        &self,
        channel_id: impl Display,
        message_id: impl Display,
    ) -> RequestBuilder {
        self.0.patch(format!(
            "{}/channels/{}/messages/{}",
            Self::BASE_URL,
            channel_id,
            message_id
        ))
    }

    pub fn delete_message(
        &self,
        channel_id: impl Display,
        message_id: impl Display,
    ) -> RequestBuilder {
        self.0.delete(format!(
            "{}/channels/{}/messages/{}",
            Self::BASE_URL,
            channel_id,
            message_id
        ))
    }

    pub fn interaction_response(
        &self,
        interaction_id: impl Display,
        interaction_token: impl Display,
    ) -> RequestBuilder {
        self.0.post(format!(
            "{}/interactions/{}/{}/callback",
            Self::BASE_URL,
            interaction_id,
            interaction_token
        ))
    }

    pub fn bulk_delete(&self, channel_id: impl Display) -> RequestBuilder {
        self.0.post(format!(
            "{}/channels/{}/messages/bulk-delete",
            Self::BASE_URL,
            channel_id
        ))
    }

    pub async fn execute_request(
        &self,
        trace_id: TraceId,
        request: reqwest::RequestBuilder,
    ) -> Result<Response, reqwest::Error> {
        #[derive(serde::Deserialize)]
        struct RateLimited {
            retry_after: f32,
        }

        impl From<RateLimited> for std::time::Duration {
            fn from(value: RateLimited) -> Self {
                Duration::from_secs_f32(value.retry_after)
            }
        }
        let response = loop {
            let copy = request.try_clone().unwrap();

            let request = copy.build()?;
            let tx = sentry::start_transaction(TransactionContext::new_with_trace_id(
                format!("{} {}", request.method(), request.url()).as_str(),
                "http.client",
                trace_id,
            ));
            tx.set_tag("http.method", request.method());
            tx.set_request(sentry::protocol::Request {
                url: Some(request.url().clone()),
                method: Some(request.method().to_string()),
                cookies: request
                    .headers()
                    .get("cookie")
                    .map(|f| f.to_str().unwrap().to_string()),
                ..Default::default()
            });
            let response = self.0.execute(request).await?;
            tx.set_tag("http.status_code", response.status().as_u16());
            if response.status() == 429 {
                let retry: RateLimited = response.json().await?;
                info!(
                    "Got Rate limited, waiting for {} seconds.",
                    retry.retry_after
                );
                let span = tx.start_child("rate_limit.wait", "Waiting for Rate Limits");
                tx.set_tag("rate_limited", true);
                span.set_data("retry_after", retry.retry_after.into());
                tokio::time::sleep(retry.into()).await;
                span.finish();
                tx.set_status(SpanStatus::Ok);
                tx.finish();
                continue;
            }
            tx.set_tag("rate_limited", false);
            tx.set_status(SpanStatus::Ok);
            tx.finish();
            break response;
        };

        Ok(response)
    }
}
