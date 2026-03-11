use crate::{TimeInterval, UrlError};
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

pub fn get_host(url: &str) -> Result<String, UrlError> {
    let url = Url::parse(url);

    match url {
        Ok(url) => {
            let host = url.host_str().map(|h| h.to_string());
            host.ok_or(UrlError::InvalidHost)
        }
        Err(_) => Err(UrlError::InvalidUrlPath),
    }
}
