use crate::configs::{ConfigWithClock, HostConfig, GlobalConfig};
use crate::helpers::get_host;
use configs::Config;
use dashmap::DashMap;
use governor::RateLimiter;
use governor::clock::{Clock, DefaultClock, Reference};
use helpers::build_quota;
pub use models::{TimeInterval, UrlError};
use types::DirectLimiter;

pub mod configs;
mod helpers;
mod models;
pub mod types;

/// `RateLimitClient` is the main type of the library. It contains a global config and a record
/// of all registered hosts.
///
/// See `build` method for more details.
#[derive(Debug)]
pub struct RateLimitClient<C: Clock + Clone = DefaultClock> {
    config: GlobalConfig<C>,
    hosts: DashMap<String, Host<C>>,
}


#[derive(Debug)]
struct Host<C: Clock + Clone> {
    quota: DirectLimiter<C>,
}

impl RateLimitClient<DefaultClock> {
    /// Builds a client with the specified `quota`, `burst`, and `time`. It is the most common way to
    /// create a client.
    ///
    /// `quota` is a `NonZeroU32` type used to determine the interval of "ticket" recovery.
    /// Suppose you build a client with a quota of `10`, a burst of `1`, a `time` of
    /// `ByMinutes`, which means that you can make one request every 6 minutes.
    ///
    /// The `burst` determines how many tickets that can stack. If you
    /// set `burst` to `2`, up to two request can be made immediatelly.

    /// After a specific time interval is finished, you can perform another `1` operation.
    ///
    /// Let's consider the example from the `quota` section description where
    /// our client recovers one ticket every 6 minutes. With a burst of `2`,
    /// it will request two at
    /// same time, then each 6 minutes, you can perform other request. If the client is idle by at
    /// least 12 minutes, when you do a request, it will be two at same time again.
    ///
    /// `time` is the interval meansurement used along of `quota` for determine the interval. It's
    /// something like `quota`/`time`, but it's more complex than it, of course.
    ///
    /// For more information see [Generic Cell Rate Algorithm](https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm)
    ///
    /// Example of how to use:
    /// ```rust
    /// use std::num::NonZeroU32;
    /// use http_client::{RateLimitClient, TimeInterval};
    /// use http_client::configs::Config;
    ///
    /// fn main() {
    ///     let config = Config {
    ///        quota: NonZeroU32::new(2).unwrap(),
    ///        burst: NonZeroU32::new(1).unwrap(),
    ///        interval: TimeInterval::ByHours,
    ///     };
    ///     let client = RateLimitClient::build(config);
    /// }
    /// ```
    pub fn build(config: Config) -> Self {
        let config = GlobalConfig::build(config);

        Self {
            config,
            hosts: DashMap::new(),
        }
    }

    /// Perform a get request. If `url` is a invalid *url* format, it will panic.
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, reqwest::Error> {
        let host = get_host(url).unwrap_or_else(|_| panic!("Invalid Url Format: {}", url));

        match self.hosts.get(&host) {
            Some(host) => host.quota.until_ready().await,
            None => {
                self.config
                    .limit
                    .until_key_ready(&"global".to_string())
                    .await
            }
        }

        self.config.client.get(url).send().await
    }
}

impl<C> RateLimitClient<C>
where
    C: Clock + Clone,
    C::Instant: Reference,
{
    /// Builds a client with a clock. It's common in testing scenarios. It's strongly recommended
    /// to build a client using `build`.
    pub fn build_with_clock(config: ConfigWithClock<C>) -> Self {
        let config = GlobalConfig::build_with_clock(config);

        Self {
            config,
            hosts: DashMap::new(),
        }
    }

    /// Check if there are some available "ticket" for usage in global hosts.
    ///
    /// > This method will be removed in the future.
    pub fn global_limit_is_ok(&self, key: &str) -> bool {
        self.config.limit.check_key(&key.to_string()).is_ok()
    }

    /// Check if there are not some available "ticket" for usage in global hosts.
    ///
    /// > This method will be removed in the future.
    pub fn global_limit_is_err(&self, key: &str) -> bool {
        self.config.limit.check_key(&key.to_string()).is_err()
    }

    /// Check if there are some ticket in a specific host.
    pub fn host_limit_is_ok(&self, key: &str) -> bool {
        let host = get_host(key).expect("Invalid Hostname format");
        let host = self.hosts.get(&host).expect("Host actually to exist");

        host.quota.check().is_ok()
    }

    /// Builds a host with a separated quota. You must pass a valid hostname, not a *url*.
    /// Example of usage:
    ///
    /// ```rust
    /// use std::num::NonZeroU32;
    /// use http_client::{
    ///     RateLimitClient, 
    ///     TimeInterval,
    ///     configs::{Config, HostConfig},
    /// };
    /// 
    /// fn main() {
    ///     
    ///     let client = RateLimitClient::build(Config {
    ///        quota: NonZeroU32::new(2).unwrap(),
    ///        burst: NonZeroU32::new(1).unwrap(),
    ///        interval: TimeInterval::ByHours,
    ///     });
    ///
    ///     client.build_host(HostConfig { 
    ///             base: Config {
    ///                 quota: NonZeroU32::new(20).unwrap(),
    ///                 burst: NonZeroU32::new(2).unwrap(),
    ///                 interval: TimeInterval::ByHours, 
    ///             },
    ///             hostname: "google.com", 
    ///      });
    /// }
    /// ```
    pub fn build_host(
        &self,
        config: HostConfig
    ) {
        let quota = build_quota(config.quota, config.burst, config.interval);
        let limit = RateLimiter::direct_with_clock(quota, self.config.clock.clone());
        let host_config = Host { quota: limit };

        self.hosts.insert(config.hostname.to_string(), host_config);
    }

    /// Checks if a host exists
    pub fn host_exists(&self, host: &str) -> bool {
        self.hosts.contains_key(host)
    }
}
