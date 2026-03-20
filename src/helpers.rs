use crate::{TimeInterval, errors::HttpClientError};
use governor::Quota;
use std::num::NonZeroU32;
use url::Url;

pub fn build_quota(quota: NonZeroU32, burst: NonZeroU32, interval: TimeInterval) -> Quota {
    match interval {
        TimeInterval::ByHours => Quota::per_hour(quota).allow_burst(burst),
        TimeInterval::ByMinutes => Quota::per_minute(quota).allow_burst(burst),
        TimeInterval::BySeconds => Quota::per_second(quota).allow_burst(burst),
    }
}

pub fn get_host(url: &str) -> Result<String, HttpClientError> {
    let url_parsed = Url::parse(url);

    match url_parsed {
        Ok(u) => {
            let host = u.host_str().map(|h| h.to_string());
            host.ok_or(HttpClientError::NoHostname(url.to_string()))
        }
        Err(e) => Err(HttpClientError::ParseHostError(e)),
    }
}
