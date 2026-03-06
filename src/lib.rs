use governor::clock::{Clock, DefaultClock, Reference};
use governor::middleware::NoOpMiddleware;
use governor::state::keyed::DashMapStateStore;
use governor::state::{InMemoryState, NotKeyed};
use governor::RateLimiter;
use dashmap::DashMap;
use reqwest::Client;
use tracing::{instrument, info};
use std::num::NonZeroU32;
use helpers::build_quota;
use crate::helpers::get_host;
pub use models::{TimeInterval, UrlError};

mod helpers;
mod models;

type Middleware<C> = NoOpMiddleware<<C as Clock>::Instant>;
type DirectLimiter<C> = RateLimiter<NotKeyed, InMemoryState, C, Middleware<C>>;
type KeyedLimiter<C> = RateLimiter<String, DashMapStateStore<String>, C, Middleware<C>>;

// TODO: RateLimitClient is too important in the program what any change could
// break out the whole application. It's possible separete responsibility 
// with Host too, especially the "hosts" fields. 
// 
// I idea is separate Host from Global Config. Maybe a HostConfig could be 
// possible for coeherence.
// 
// It probably will cost a lot of time and handche. 
#[derive(Debug)]
pub struct RateLimitClient<C: Clock + Clone = DefaultClock> {
    config: GlobalConfig<C>,
    hosts: DashMap<String, Host<C>>,
}

#[derive(Debug)]
struct GlobalConfig<C: Clock + Clone> {
    limit: KeyedLimiter<C>,
    client: Client,
    clock: C, 
}

impl GlobalConfig<DefaultClock> {
    pub fn build(quota: NonZeroU32, time: TimeInterval) -> Self {
        GlobalConfig::build_with_clock(quota, time, DefaultClock::default())
    }
}

impl<C> GlobalConfig<C> 
where
    C: Clock + Clone,
    C::Instant: Reference,
{
    pub fn build_with_clock(quota: NonZeroU32, time: TimeInterval, clock: C) -> Self {
        let limit = RateLimiter::new(
            build_quota(quota, time),
            DashMapStateStore::default(),
            clock.clone(),
        );
        
        Self {
            limit,
            client: Client::new(),
            clock, 
        }
    }
}

#[derive(Debug)]
struct HostConfig<C: Clock + Clone = DefaultClock> {
    limit: DirectLimiter<C>,
}

#[derive(Debug)]
struct Host<C: Clock + Clone> {
    config: HostConfig<C>,
}

impl RateLimitClient<DefaultClock> {
    pub fn build(quota: NonZeroU32, time: TimeInterval) -> Self {
        let config = GlobalConfig::build(quota, time);

        Self {
            config,
            hosts: DashMap::new(),
        }
    }
    
    #[instrument(skip(self))]
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        match self.hosts.get(&url.to_string()) {
            Some(host) => host.config.limit.until_ready().await,
            None => self.config.limit.until_key_ready(&"global".to_string()).await,
        }
        
        info!("request started");
        self.config.client.get(url).send().await
    }
}

impl<C> RateLimitClient<C>
where
    C: Clock + Clone,
    C::Instant: Reference,
{
    pub fn build_with_clock(quota: NonZeroU32, time: TimeInterval, clock: C) -> Self {
        let config = GlobalConfig::build_with_clock(quota, time, clock);

        Self {
            config,
            hosts: DashMap::new(),
        }
    }

    pub fn global_limit_is_ok(&self, key: &str) -> bool {
        self.config.limit.check_key(&key.to_string()).is_ok()
    }

    pub fn global_limit_is_err(&self, key: &str) -> bool {
        self.config.limit.check_key(&key.to_string()).is_err()        
    }
    
    pub fn host_limit_is_ok(&self, key: &str) -> bool {
        let host = get_host(key).expect("Invalid Hostname format");
        let host = self.hosts.get(&host).expect("Host actually to exist");
        
        host.config.limit.check().is_ok()
    }
    
    pub fn host_limit_is_err(&self, key: &str) -> bool {
        let host = get_host(key).expect("Invalid Hostname format");
        let host = self.hosts.get(&host).expect("Host actually to exist");
        
        host.config.limit.check().is_err()
    }

    pub fn build_host(&mut self, host: &str, quota: NonZeroU32, interval: TimeInterval) {
        let quota = build_quota(quota, interval);
        let limit = RateLimiter::direct_with_clock(quota, self.config.clock.clone());
        let config = HostConfig { limit };
        let host_config = Host { config };

        self.hosts.insert(host.to_string(), host_config);
    }
    
    pub fn host_exists(&self, host: &str) -> bool {
        self.hosts.contains_key(host) 
    }
    
 
}