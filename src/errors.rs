use std::fmt;

#[derive(Debug)]
pub enum HttpClientError {
    ParseHostError(url::ParseError),
    HostNotFound(String),
    Request(reqwest::Error),
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

impl From<reqwest::Error> for HttpClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::Request(e)
    }
}