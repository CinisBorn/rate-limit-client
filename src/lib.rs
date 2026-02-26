use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use reqwest::Client;
use std::num::NonZeroU32;

pub enum TimeMeasurement {
    BySeconds,
    ByMinutes,
    ByHours,
}

pub struct ClientBuilder {
    limit: DefaultDirectRateLimiter,
    client: Client,
}

pub struct Host {
    config: ClientBuilder,
}

impl Host {
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        self.config.limit.until_ready().await;
        self.config.client.get(url).send().await
    }
}

impl ClientBuilder {
    pub fn build(quota: NonZeroU32, interval: TimeMeasurement) -> Host {
        let burst = NonZeroU32::new(1).expect("No Zero Expected");
        
        let quota_limit = match interval {
            TimeMeasurement::ByHours => Quota::per_hour(quota).allow_burst(burst),
            TimeMeasurement::ByMinutes => Quota::per_minute(quota).allow_burst(burst),
            TimeMeasurement::BySeconds => Quota::per_second(quota).allow_burst(burst),
        };
        
        let limit = RateLimiter::direct(quota_limit);
        
        Host { config: Self { client: Client::new(), limit }}
    }
}
