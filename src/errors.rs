//! This module contains every error structs for error handling. 
use std::fmt;
use std::error::Error;
/// Error for client operations.
#[derive(Debug)]
pub enum HttpClientError {
    /// Trigger if the url passed is invalid
    ParseHostError(url::ParseError),
    /// Trigger if host is not registered when `host_get` method is used.
    HostNotFound(String),
    /// Trigger when an error occurs during the request.
    Request(reqwest::Error),
    /// Trigger if it's not possible extract the hostname from a url
    /// when `host_get` method is used.
    NoHostname(String)
}

impl fmt::Display for HttpClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpClientError::HostNotFound(e) => write!(f, "Host not registred: {e}"),
            HttpClientError::ParseHostError(e) => write!(f, "Invalid url: {e}"),
            HttpClientError::Request(e) => write!(f, "Request Failed: {e}"),
            HttpClientError::NoHostname(e) => write!(f, "The url does not have a host: {e}")
        }
    }
}

impl Error for HttpClientError {}

impl From<reqwest::Error> for HttpClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}