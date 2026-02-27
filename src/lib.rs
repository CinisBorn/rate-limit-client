use governor::{RateLimiter, Quota};
use governor::clock::{Clock, DefaultClock, ReasonablyRealtime, Reference};
use governor::middleware::NoOpMiddleware;
use governor::state::{NotKeyed, InMemoryState};
use governor::state::keyed::{DashMapStateStore};

use std::{collections::HashMap, num::NonZeroU32};
use reqwest::Client;

type Middleware<C> = NoOpMiddleware<<C as Clock>::Instant>;
type DirectLimiter<C> = governor::RateLimiter<NotKeyed, InMemoryState, C, Middleware<C>>;
type KeyedLimiter<C> = governor::RateLimiter<String, DashMapStateStore<String>, C, Middleware<C>>;

pub enum TimeInterval {
    BySeconds,
    ByMinutes,
    ByHours,
}

pub struct RateLimitClient<C: Clock + Clone = DefaultClock> {
    client: Client,
    clock: C,                         // stored so add_host can clone it
    hosts: HashMap<String, Host<C>>,
    default_limit: KeyedLimiter<C>,
}

struct Host<C: Clock + Clone> {
    limit: DirectLimiter<C>,
}

impl RateLimitClient<DefaultClock> {
    pub fn build_default() -> Self {
        let burst = NonZeroU32::new(10).expect("No Zero Burst");
        let quota = governor::Quota::per_second(burst);
        let limit = governor::RateLimiter::keyed(quota);
        
        Self {
            client: Client::new(),
            default_limit: limit,
            clock: DefaultClock::default(),
            hosts: HashMap::new()
        }
    }
}

impl<C> RateLimitClient<C> 
where
    C: Clock + Clone + ReasonablyRealtime,
    C::Instant: Reference, 
{
    pub fn build_with_clock(clock: C, quota: NonZeroU32, interval: TimeInterval) -> Self {
        let burst = NonZeroU32::new(1).expect("Be the number 1");
        let limit = match interval {
            TimeInterval::ByHours => {
                let quota = Quota::per_hour(quota).allow_burst(burst);
                RateLimiter::new(quota, DashMapStateStore::default(), clock.clone())
            },
            TimeInterval::ByMinutes => {
                let quota = Quota::per_minute(quota).allow_burst(burst);
                RateLimiter::new(quota, DashMapStateStore::default(), clock.clone())
            },
            TimeInterval::BySeconds => {
                let quota = Quota::per_second(quota).allow_burst(burst);
                RateLimiter::new(quota, DashMapStateStore::default(), clock.clone())
            }
        };
        
        Self {
            client: Client::new(),
            clock,
            hosts: HashMap::new(),
            default_limit: limit,
        }
    }
    
    pub fn build_host(mut self, host: &str, quota: NonZeroU32, interval: TimeInterval) -> Self {
        let burst = NonZeroU32::new(1).expect("Be the number 1");
        let limit = match interval {
            TimeInterval::ByHours => {
                let quota = Quota::per_hour(quota).allow_burst(burst);
                RateLimiter::direct_with_clock(quota, self.clock.clone())
            },
            TimeInterval::ByMinutes => {
                let quota = Quota::per_minute(quota).allow_burst(burst);
                RateLimiter::direct_with_clock(quota, self.clock.clone())
            },
            TimeInterval::BySeconds => {
                let quota = Quota::per_second(quota).allow_burst(burst);
                RateLimiter::direct_with_clock(quota, self.clock.clone())
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
            },
            None => {
                self.default_limit.until_key_ready(&String::from("global")).await;
            }
        }
        
        self.client.get(url).send().await
    }
}