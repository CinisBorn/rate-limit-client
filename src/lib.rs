use governor::clock::{Clock, DefaultClock, Reference};
use governor::middleware::NoOpMiddleware;
use governor::state::keyed::DashMapStateStore;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use dashmap::DashMap;
use reqwest::Client;
use std::num::NonZeroU32;
use helpers::build_quota;
use crate::helpers::get_host;
pub use models::{TimeInterval, UrlError};

mod helpers;
mod models;

const GLOBAL_KEY: &'static str = "global";

type Middleware<C> = NoOpMiddleware<<C as Clock>::Instant>;
type DirectLimiter<C> = RateLimiter<NotKeyed, InMemoryState, C, Middleware<C>>;
type KeyedLimiter<C> = RateLimiter<String, DashMapStateStore<String>, C, Middleware<C>>;

pub struct RateLimitClient<C: Clock + Clone = DefaultClock> {
    client: Client,
    clock: C, // stored so add_host can clone it
    hosts: DashMap<String, Host<C>>,
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
            hosts: DashMap::new(),
        }
    }

    pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        match self.hosts.get(&url.to_string()) {
            Some(host) => host.limit.until_ready().await,
            None => self.default_limit.until_key_ready(&GLOBAL_KEY.to_string()).await,
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
            clock.clone(),
        );

        Self {
            client: Client::new(),
            clock,
            hosts: DashMap::new(),
            default_limit: limit,
        }
    }

    pub fn get_default_limit(&self) -> &KeyedLimiter<C> {
        &self.default_limit
    }

    pub fn host_limit_is_ok(&self, key: &str) -> bool {
        let host = get_host(key).expect("Invalid Hostname format");
        let host = self.hosts.get(&host).expect("Host actually to exist");
        
        host.limit.check().is_ok()
    }
    
    pub fn host_limit_is_err(&self, key: &str) -> bool {
        let host = get_host(key).expect("Invalid Hostname format");
        let host = self.hosts.get(&host).expect("Host actually to exist");
        
        host.limit.check().is_err()
    }

    pub fn build_host(&mut self, host: &str, quota: NonZeroU32, interval: TimeInterval) {
        let quota = build_quota(quota, interval);
        let limit = RateLimiter::direct_with_clock(quota, self.clock.clone());
        let host_config = Host { limit };

        self.hosts.insert(host.to_string(), host_config);
    }
}
