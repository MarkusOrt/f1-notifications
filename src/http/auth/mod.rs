use std::convert::Infallible;

use axum::{
    extract::{FromRequestParts, Query, State},
    http::{HeaderMap, HeaderValue},
    response::Redirect,
};
use axum_extra::headers::{Cookie, HeaderMapExt};
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use libsql::params;
use rand::RngExt;
use reqwest::{
    StatusCode,
    header::{LOCATION, SET_COOKIE},
};

use crate::{USER_AGENT, error::Error, http::AxumState};

#[cfg(debug_assertions)]
const REDIRECT_URI: &str = "http://127.0.0.1:8123/post-auth";
#[cfg(not(debug_assertions))]
const REDIRECT_URI: &str = "https://f1-notifications.ort.dev/post-auth";

#[derive(serde::Deserialize)]
pub struct User {
    pub id: i64,
    #[serde(alias = "name")]
    pub username: String,
    #[serde(alias = "discord_id")]
    pub user_id: String,
    #[serde(alias = "discord_token")]
    pub token: String,
    pub created_at: DateTime<Utc>,
}

impl FromRequestParts<AxumState<'_>> for User {
    type Rejection = crate::error::Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AxumState<'_>,
    ) -> Result<Self, Self::Rejection> {
        let Some(cookie) = parts.headers.typed_get::<Cookie>() else {
            return Err(Error::Unauthorized);
        };

        let Some(session_token) = cookie.get("session") else {
            return Err(Error::Unauthorized);
        };

        let data = get_user_from_session(&state.db_pool, session_token)
            .await
            .inspect_err(|e| println!("{e}"))?;
        if let Some(data) = data {
            Ok(data)
        } else {
            Err(crate::error::Error::Unauthorized)
        }
    }
}

impl FromRequestParts<AxumState<'_>> for Option<User> {
    type Rejection = Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AxumState<'_>,
    ) -> Result<Self, Self::Rejection> {
        Ok(User::from_request_parts(parts, state).await.ok())
    }
}

pub async fn get_user_from_session(
    db: &libsql::Connection,
    session_token: &str,
) -> Result<Option<User>, crate::error::Error> {
    let mut cursor = db.query("SELECT u.* FROM user_sessions s JOIN users u ON user_id = u.id WHERE token = ? AND expires_at > CURRENT_TIMESTAMP", 
        params![session_token]).await?;
    Ok(match cursor.next().await? {
        Some(row) => Some(libsql::de::from_row(&row)?),
        None => None,
    })
}

pub async fn create_auth_token(
    db: &libsql::Connection,
    token: &str,
    expiry: DateTime<Utc>,
) -> Result<(), crate::error::Error> {
    let result = db
        .execute(
            "INSERT INTO auth_sessions (token, expires_at, created_at) VALUES (?, ?, ?)",
            params![
                token,
                expiry.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            ],
        )
        .await?;
    _ = result;
    Ok(())
}

pub async fn auth(
    user: Option<User>,
    State(app_state): State<AxumState<'_>>,
) -> Result<Redirect, crate::error::Error> {
    if user.is_some() {
        return Ok(Redirect::temporary("/"));
    }
    let mut token = [0u8; 32];
    rand::rng().fill(&mut token);
    let expiry = Utc::now() + Duration::minutes(5);
    let digest = sha256::digest(&token);
    create_auth_token(&app_state.db_pool, &digest, expiry)
        .await
        .inspect_err(|f| println!("{f:?}"))?;
    let uri = format!(
        "https://discord.com/oauth2/authorize?client_id=942121950778634240&response_type=code&redirect_uri={}&scope=identify&state={}",
        uri_encode::encode_uri_component(REDIRECT_URI),
        digest
    );
    Ok(Redirect::temporary(&uri))
}

#[derive(serde::Deserialize)]
pub struct QueryParams {
    code: String,
    state: String,
}

#[derive(serde::Serialize)]
pub struct CodeSender {
    grant_type: &'static str,
    code: String,
    redirect_uri: String,
}

#[allow(dead_code)]
#[derive(serde::Deserialize)]
pub struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: i32,
    refresh_token: String,
    scope: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct MeResponse {
    user: DiscordUser,
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
pub struct DiscordUser {
    id: String,
    username: String,
    global_name: String,
}

pub async fn post_auth(
    State(app_state): State<AxumState<'_>>,
    Query(query_data): Query<QueryParams>,
) -> Result<(StatusCode, HeaderMap), crate::error::Error> {
    let client_id = std::env::var("CLIENT_ID")?;
    let client_secret = std::env::var("CLIENT_SECRET")?;
    auth_session_check(&app_state.db_pool, &query_data.state)
        .await
        .inspect_err(|f| println!("{f}"))?;

    let client = reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?;

    let token_response = client
        .post("https://discord.com/api/v10/oauth2/token")
        .basic_auth(client_id, Some(client_secret))
        .form(&CodeSender {
            grant_type: "authorization_code",
            code: query_data.code,
            redirect_uri: REDIRECT_URI.to_owned(),
        })
        .send()
        .await
        .inspect_err(|f| println!("{f}"))?
        .json::<TokenResponse>()
        .await
        .inspect_err(|f| println!("{f}"))?;

    let me = client
        .get("https://discord.com/api/v10/oauth2/@me")
        .bearer_auth(token_response.access_token)
        .send()
        .await
        .inspect_err(|f| println!("{f}"))?
        .json::<MeResponse>()
        .await
        .inspect_err(|f| println!("{f}"))?;

    if !check_user_id(&me.user.id) {
        return Err(crate::error::Error::Unauthorized);
    }

    let user_id = create_user(
        &app_state.db_pool,
        &me.user.username,
        &me.user.id,
        &token_response.refresh_token,
    )
    .await
    .inspect_err(|f| println!("{f}"))?;

    let mut session_token = [0u8; 32];
    rand::rng().fill(&mut session_token);
    let session_token = base64::prelude::BASE64_STANDARD.encode(session_token);

    create_session_for_user(&app_state.db_pool, user_id, &session_token)
        .await
        .inspect_err(|f| println!("{f}"))?;

    let expiry = Utc::now() + Duration::days(7);
    let cookie = format!(
        "session={}; Expires={}; Path=/; HttpOnly; Secure",
        session_token,
        expiry.format("%a, %d %b %Y %H:%M:%S GMT")
    );

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.parse().unwrap());
    headers.insert(LOCATION, HeaderValue::from_static("/"));

    Ok((StatusCode::TEMPORARY_REDIRECT, headers))
}

async fn auth_session_check(
    db: &libsql::Connection,
    state: &str,
) -> Result<(), crate::error::Error> {
    let mut cursor = db
        .query(
            "SELECT * FROM auth_sessions WHERE token = ?",
            params![state],
        )
        .await?;
    match cursor.next().await? {
        Some(_) => Ok(()),
        None => Err(crate::error::Error::Unauthorized),
    }
}

async fn create_user(
    db: &libsql::Connection,
    username: &str,
    user_id: &str,
    token: &str,
) -> Result<u64, libsql::Error> {
    db.execute(
        "INSERT INTO users (name, discord_id, discord_token, created_at) VALUES (?, ?, ?, ?)",
        params![
            username,
            user_id,
            token,
            Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        ],
    )
    .await
}

fn check_user_id(id: &str) -> bool {
    for i in [
        "142951266811641856", // Markus
        "111928351798636544", // Ren
        "186051153254023168", // Coco
        "61155535751225344",  // Azumao
        "227851317136195584", // Slendis
        "260058592852443149", // Redacted
        "265827741847257089", // Blue,
        "86939871179980800",  // Teeteegone
        "310338524891185153", // RebII
        "144476078377795584", // GalacticHitchHiker
        "808002254710898708", // Gucks
    ]
    .iter()
    {
        if *i == id {
            return true;
        }
    }
    false
}

async fn create_session_for_user(
    db: &libsql::Connection,
    user_id: u64,
    session_token: &str,
) -> Result<(), crate::error::Error> {
    let now = Utc::now();
    db.execute(
        "INSERT INTO user_sessions (user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?)",
        params![
            user_id,
            session_token,
            (now + chrono::Duration::days(7)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            now.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        ],
    )
    .await?;
    Ok(())
}
