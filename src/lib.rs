use governor::{DefaultDirectRateLimiter, DefaultKeyedRateLimiter, Quota, RateLimiter};
use std::{collections::HashMap, num::NonZeroU32};
use reqwest::Client;

pub enum TimeInterval {
    BySeconds,
    ByMinutes,
    ByHours,
}

pub struct RateLimitClient {
    client: Client,
    hosts: HashMap<String, Host>,
    default_limit: DefaultKeyedRateLimiter<String>
}

struct Host {
    limit: DefaultDirectRateLimiter,
}

impl RateLimitClient {
    pub fn build_default() -> Self {
        let burst = NonZeroU32::new(10).expect("No Zero Burst");
        let quota = Quota::per_second(burst);
        let limit = RateLimiter::keyed(quota);
        
        Self {
            client: Client::new(),
            default_limit: limit,
            hosts: HashMap::new()
        }
    }
    
    pub fn build(quota: NonZeroU32, interval: TimeInterval) -> Self {
        let burst = NonZeroU32::new(1).expect("Be the number 1");
        let limit = match interval {
            TimeInterval::ByHours => {
                RateLimiter::keyed(Quota::per_hour(quota).allow_burst(burst))
            },
            TimeInterval::ByMinutes => {
                RateLimiter::keyed(Quota::per_minute(quota).allow_burst(burst))
            },
            TimeInterval::BySeconds => {
                RateLimiter::keyed(Quota::per_second(quota).allow_burst(burst))
            }
        };
        
        Self {
            client: Client::new(),
            default_limit: limit,
            hosts: HashMap::new()
        }
    }
    
    pub fn build_host(mut self, host: &str, quota: NonZeroU32, interval: TimeInterval) -> Self {
        let burst = NonZeroU32::new(1).expect("Be the number 1");
        let limit = match interval {
            TimeInterval::ByHours => {
                RateLimiter::direct(Quota::per_hour(quota).allow_burst(burst))
            },
            TimeInterval::ByMinutes => {
                RateLimiter::direct(Quota::per_minute(quota).allow_burst(burst))
            },
            TimeInterval::BySeconds => {
                RateLimiter::direct(Quota::per_second(quota).allow_burst(burst))
            }
        };
        let host_config = Host { 
            limit 
        };
        
        self.hosts.insert(host.to_string(), host_config);
        
        self 
    }
    
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        match self.hosts.get(&url.to_string()) {
            Some(host) => {
                host.limit.until_ready().await;
                self.client.get(url).send().await
            },
            None => {
                self.default_limit.until_key_ready(&String::from("global")).await;
                self.client.get(url).send().await
            }
        }
    }
}