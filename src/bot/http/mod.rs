#![allow(dead_code)]
//! HTTP Client Handler

use std::fmt::Display;

use axum::http::HeaderMap;
use reqwest::{RequestBuilder, header::AUTHORIZATION};

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
}
