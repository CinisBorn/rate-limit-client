use governor::{RateLimiter, Quota};
use governor::clock::{Clock, DefaultClock, Reference};
use governor::middleware::NoOpMiddleware;
use governor::state::{NotKeyed, InMemoryState};
use governor::state::keyed::{DashMapStateStore};

use std::{collections::HashMap, num::NonZeroU32};
use reqwest::Client;

use helpers::build_quota;

mod helpers;

type Middleware<C> = NoOpMiddleware<<C as Clock>::Instant>;
type DirectLimiter<C> = RateLimiter<NotKeyed, InMemoryState, C, Middleware<C>>;
type KeyedLimiter<C> = RateLimiter<String, DashMapStateStore<String>, C, Middleware<C>>;

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
        let quota = Quota::per_second(burst);
        let limit = RateLimiter::keyed(quota);
        
        Self {
            client: Client::new(),
            default_limit: limit,
            clock: DefaultClock::default(),
            hosts: HashMap::new()
        }
    }
    
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let global_key = &String::from("global");
        
        match self.hosts.get(&url.to_string()) {
            Some(host) => host.limit.until_ready().await,
            None => self.default_limit.until_key_ready(global_key).await,
        }
        
        self.client.get(url).send().await
    }
}

impl<C> RateLimitClient<C> 
where
    C: Clock + Clone,
    C::Instant: Reference, 
{
    pub fn build_with_clock(clock: C, quota: NonZeroU32, interval: TimeInterval) -> Self {
        let limit = RateLimiter::new(
            build_quota(quota, interval), 
            DashMapStateStore::default(), 
            clock.clone()
        );
        
        Self {
            client: Client::new(),
            clock,
            hosts: HashMap::new(),
            default_limit: limit,
        }
    }
    
    pub fn get_default_limit(&self) -> &KeyedLimiter<C> {
        &self.default_limit
    }
    
    pub fn get_host_limit(&self, key: &str) -> Result<&DirectLimiter<C>, ()> {
        match self.hosts.get(key) {
            Some(k) => Ok(&k.limit),
            None => Err(())
        }
    }
    
    pub fn build_host(mut self, host: &str, quota: NonZeroU32, interval: TimeInterval) -> Self {
        let limit = RateLimiter::direct_with_clock(
            build_quota(quota, interval), 
            self.clock.clone()
        );
        let host_config = Host { limit };
        
        self.hosts.insert(host.to_string(), host_config);
        
        self 
    }
}

