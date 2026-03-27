#![allow(unused)]
use std::error::Error as StdError;

use axum::{
    body::Body,
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use reqwest::{StatusCode, header::CONTENT_TYPE};

pub mod form_validation;

pub use form_validation::FormValidation;

#[derive(Debug)]
pub enum Error {
    Http(reqwest::Error),
    Serde(serde::de::value::Error),
    Libsql(libsql::Error),
    Fmt(std::fmt::Error),
    ParseInt(std::num::ParseIntError),
    ParseFloat(std::num::ParseFloatError),
    Io(std::io::Error),
    FromHex(hex::FromHexError),
    Crypto(ed25519_dalek::ed25519::Error),
    Axum(axum::Error),
    Send(tokio::sync::broadcast::error::SendError<()>),
    Join(tokio::task::JoinError),
    Var(std::env::VarError),
    NotFound,
    Unauthorized,
    Form(FormValidation),
}

pub type ErrResult<T = ()> = Result<T, Error>;

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Http(error) => Some(error),
            Error::Serde(error) => Some(error),
            Error::Libsql(error) => Some(error),
            Error::Fmt(error) => Some(error),
            Error::ParseInt(error) => Some(error),
            Error::ParseFloat(error) => Some(error),
            Error::Io(error) => Some(error),
            Error::FromHex(error) => Some(error),
            Error::Crypto(error) => Some(error),
            Error::Axum(error) => Some(error),
            Error::Send(error) => Some(error),
            Error::Join(error) => Some(error),
            Error::Var(error) => Some(error),
            Error::NotFound => None,
            Error::Unauthorized => None,
            Error::Form(e) => Some(e),
        }
    }

    fn cause(&self) -> Option<&dyn StdError> {
        self.source()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Http(error) => write!(f, "{error}"),
            Error::Serde(error) => write!(f, "{error}"),
            Error::Libsql(error) => write!(f, "{error}"),
            Error::Fmt(error) => write!(f, "{error}"),
            Error::ParseInt(error) => write!(f, "{error}"),
            Error::ParseFloat(error) => write!(f, "{error}"),
            Error::Io(error) => write!(f, "{error}"),
            Error::FromHex(error) => write!(f, "{error}"),
            Error::Crypto(error) => write!(f, "{error}"),
            Error::Axum(error) => write!(f, "{error}"),
            Error::Send(error) => write!(f, "{error}"),
            Error::Join(error) => write!(f, "{error}"),
            Error::Var(error) => write!(f, "{error}"),
            Error::NotFound => write!(f, "Not Found"),
            Error::Unauthorized => write!(f, "Unauthorized"),
            Error::Form(e) => write!(f, "{e}"),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Http(value)
    }
}

impl From<serde::de::value::Error> for Error {
    fn from(value: serde::de::value::Error) -> Self {
        Error::Serde(value)
    }
}

impl From<libsql::Error> for Error {
    fn from(value: libsql::Error) -> Self {
        Error::Libsql(value)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Error::Fmt(value)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Error::ParseInt(value)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(value: std::num::ParseFloatError) -> Self {
        Error::ParseFloat(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<hex::FromHexError> for Error {
    fn from(value: hex::FromHexError) -> Self {
        Error::FromHex(value)
    }
}

impl From<ed25519_dalek::ed25519::Error> for Error {
    fn from(value: ed25519_dalek::ed25519::Error) -> Self {
        Error::Crypto(value)
    }
}

impl From<axum::Error> for Error {
    fn from(value: axum::Error) -> Self {
        Error::Axum(value)
    }
}

impl From<tokio::sync::broadcast::error::SendError<()>> for Error {
    fn from(value: tokio::sync::broadcast::error::SendError<()>) -> Self {
        Error::Send(value)
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(value: tokio::task::JoinError) -> Self {
        Error::Join(value)
    }
}

impl From<std::env::VarError> for Error {
    fn from(value: std::env::VarError) -> Self {
        Error::Var(value)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("{self}");
        let mut response = if let Self::Form(e) = &self {
            let mut response =
                Response::new(Body::new(serde_json::to_string_pretty(&e.0).unwrap()));
            response
                .headers_mut()
                .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            response
        } else {
            Response::new(Body::empty())
        };
        *response.status_mut() = match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Form(_) => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        response
    }
}
