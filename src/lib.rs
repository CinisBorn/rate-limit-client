use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use std::num::{NonZeroU32};
use reqwest::Client;

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
    pub fn build(quota: NonZeroU32) -> Host {
        let burst = NonZeroU32::new(1).expect("No Zero Expected");
        let quota = Quota::per_second(quota).allow_burst(burst);
        let limit = RateLimiter::direct(quota);
        
        Host { config: Self { client: Client::new(), limit}}
    }
}