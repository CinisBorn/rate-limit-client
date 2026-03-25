//! # Description 
//! 
//! This project is a rate-limited HTTP client. It limits the number of requests 
//! made based on a configured quota and time interval.
//! 
//! ## Example
//! 
//! Let's suppose we want to get information from `api.github.com` 
//! without authentication. It has a limit of 60 requests per hour. 
//! We want to prevent 429 (Too Many Requests) errors:
//! 
//! ```no_run
//! use std::{error::Error, num::NonZeroU32};
//! use rate_limit_client::{
//!     RateLimitClient, 
//!     TimeInterval,
//!     configs::{Config, HostConfig}
//! };
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     let client = RateLimitClient::build(Config {
//!         quota: NonZeroU32::new(10).unwrap(),
//!         burst: NonZeroU32::new(2).unwrap(),
//!         interval: TimeInterval::ByMinutes,
//!     });
//!     
//!     client.build_host(HostConfig {
//!         base: Config {
//!             quota: NonZeroU32::new(60).unwrap(),
//!             burst: NonZeroU32::new(1).unwrap(),
//!             interval: TimeInterval::ByHours,
//!         },
//!         hostname: "api.github.com",
//!     });
//!     
//!     // Makes 100 requests to GitHub - respects the 60/hour limit
//!     for i in 1..=100 {
//!         let user = format!("https://api.github.com/users/user{}", i);
//!         let response = client.host_get(&user).await;
//!         
//!         match response {
//!             Ok(response) => println!("User {}: {}", i, response.status()),
//!             Err(e) => eprintln!("{}", e.to_string())
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```
//! 
//! The client will perform requests to `api.github.com` at a rate of 60 per hour,
//! while other hosts use the global limit of 10 per minute.
//! 
//! ## Configuration 
//! 
//! Configure the client with quota, burst, and time interval:
//! 
//! ```rust
//! # use rate_limit_client::{RateLimitClient, TimeInterval};
//! # use rate_limit_client::configs::Config;
//! # use std::num::NonZeroU32;
//! let client = RateLimitClient::build(Config {
//!     quota: NonZeroU32::new(10).unwrap(),
//!     burst: NonZeroU32::new(2).unwrap(),
//!     interval: TimeInterval::ByMinutes,
//! });
//! ```
//! 
//! This internally calculates the rate as `quota / interval`. 
//! In this case: 10 requests per 60 seconds = 1 request every 6 seconds.
//! 
//! The **burst** is the maximum number of tokens that can accumulate during an 
//! idle period. In this case, after 12 seconds of idle time, two tokens accumulate. 
//! When you make a request after this idle period, you can make two simultaneous 
//! requests immediately.
//! 
//! ## Host Configuration
//! 
//! Register a host with its own specific rate limit:
//! 
//! ```rust
//! # use rate_limit_client::{RateLimitClient, TimeInterval};
//! # use rate_limit_client::configs::{Config, HostConfig};
//! # use std::num::NonZeroU32;
//! # let client = RateLimitClient::build(Config {
//! #     quota: NonZeroU32::new(10).unwrap(),
//! #     burst: NonZeroU32::new(2).unwrap(),
//! #     interval: TimeInterval::ByMinutes,
//! # });
//! client.build_host(HostConfig {
//!     base: Config {
//!         quota: NonZeroU32::new(60).unwrap(),
//!         burst: NonZeroU32::new(1).unwrap(),
//!         interval: TimeInterval::ByHours,
//!     },
//!     hostname: "api.github.com",
//! });
//! ```
//! 
//! This configures rate limiting specifically for `api.github.com`.
//! 
//! ## Global Quota vs Host Quota
//! 
//! The main difference is that the **global quota** is shared among all 
//! non-registered hosts, while **host quotas** are isolated per host.
//! 
//! ### Using Global Quota
//! 
//! The `get` method uses the global quota:
//! 
//! ```no_run
//! # use rate_limit_client::{RateLimitClient, TimeInterval};
//! # use rate_limit_client::configs::Config;
//! # use std::{error::Error, num::NonZeroU32};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! # let client = RateLimitClient::build(Config {
//! #     quota: NonZeroU32::new(10).unwrap(),
//! #     burst: NonZeroU32::new(2).unwrap(),
//! #     interval: TimeInterval::ByMinutes,
//! # });
//! // Uses global quota (10 per minute)
//! for i in 1..=100 {
//!     let user = format!("https://api.github.com/users/user{}", i);
//!     let response = client.get(&user).await;
//! 
//!         match response {
//!             Ok(response) => println!("User {}: {}", i, response.status()),
//!             Err(e) => eprintln!("{}", e.to_string())
//!         }
//! }
//! # Ok(())
//! # }
//! ```
//! 
//! This makes requests at a rate of one every 6 seconds (10 per minute). 
//! However, this will eventually fail because GitHub's API has a 
//! limit of 60 requests per hour, and you will receive HTTP 429 
//! (Too Many Requests) errors.
//! 
//! ### Using Host-Specific Quota
//! 
//! The `host_get` method uses the registered host's quota:
//! 
//! ```no_run
//! # use rate_limit_client::{RateLimitClient, TimeInterval};
//! # use rate_limit_client::configs::{Config, HostConfig};
//! # use std::{error::Error, num::NonZeroU32};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! # let client = RateLimitClient::build(Config {
//! #     quota: NonZeroU32::new(10).unwrap(),
//! #     burst: NonZeroU32::new(2).unwrap(),
//! #     interval: TimeInterval::ByMinutes,
//! # });
//! # client.build_host(HostConfig {
//! #     base: Config {
//! #         quota: NonZeroU32::new(60).unwrap(),
//! #         burst: NonZeroU32::new(1).unwrap(),
//! #         interval: TimeInterval::ByHours,
//! #     },
//! #     hostname: "api.github.com",
//! # });
//! # for i in 1..=100 {
//!   # let user = format!("https://api.github.com/users/user{}", i);
//! let response = client.host_get(&user).await;
//!     #    match response {
//!     #      Ok(response) => println!("User {}: {}", i, response.status()),
//!     #      Err(e) => eprintln!("{}", e.to_string())
//!     # }
//! # }
//! # Ok(())
//! # }
//! ```
//! 
//! **Important:** If you use `get` instead of `host_get`, you will use the 
//! global quota, even for registered hosts.
//! 
//! ## Concurrent Usage
//! 
//! Wrap the client in an `Arc` to share it across multiple tasks:
//! 
//! ```no_run
//! # use rate_limit_client::{RateLimitClient, TimeInterval};
//! # use rate_limit_client::configs::Config;
//! # use std::{error::Error, num::NonZeroU32, sync::Arc};
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn Error>> {
//! # let config = Config {
//! #     quota: NonZeroU32::new(10).unwrap(),
//! #     burst: NonZeroU32::new(2).unwrap(),
//! #     interval: TimeInterval::ByMinutes,
//! # };
//! let client = Arc::new(RateLimitClient::build(config));
//! 
//! let mut handles = vec![];
//! for i in 1..=100 {
//!     let client = Arc::clone(&client);
//!     let handle = tokio::spawn(async move {
//!         let url = format!("https://api.example.com/item/{}", i);
//!         client.get(&url).await
//!     });
//!     handles.push(handle);
//! }
//! 
//! // Wait for all tasks to complete
//! for handle in handles {
//!     handle.await;
//! }
//! # Ok(())
//! # }
//! ```
//! 
//! All spawned tasks will share the same rate limiter, ensuring the total 
//! rate across all tasks respects the configured quota.
use crate::configs::{ConfigWithClock, GlobalConfig, HostConfig};
use crate::helpers::get_host;
use errors::HttpClientError;
use configs::Config;
use dashmap::DashMap;
use governor::RateLimiter;
use governor::clock::{Clock, DefaultClock, Reference};
use helpers::build_quota;
pub use models::{TimeInterval};
use types::DirectLimiter;

pub mod configs;
mod helpers;
mod models;
pub mod errors;
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
    /// Builds a client with a configuration struct, `Config`. The configuration is *global* 
    /// to all requests. See `host_build` for creating a configuration for particular hosts. 
    ///
    /// For more information see [Generic Cell Rate Algorithm](https://en.wikipedia.org/wiki/Generic_cell_rate_algorithm)
    /// # Example
    /// ```rust
    /// use std::num::NonZeroU32;
    /// use rate_limit_client::{RateLimitClient, TimeInterval};
    /// use rate_limit_client::configs::Config;
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

    /// Perform a get request. It will use the *global* quota. 
    /// # Errors 
    /// If any error occurs during the request, a `HttpClientError::Request` is returned.
    /// # Example 
    /// The client make a `get` request to `https://httpbin.org/get`.
    /// ```
    /// # use rate_limit_client::{RateLimitClient, TimeInterval};
    /// # use rate_limit_client::configs::{Config, HostConfig};
    /// # use std::num::NonZeroU32;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = RateLimitClient::build(Config {
    ///        quota: NonZeroU32::new(50).unwrap(),
    ///        burst: NonZeroU32::new(1).unwrap(),
    ///        interval: TimeInterval::ByMinutes,  
    ///     });
    /// 
    ///     client.get("https://httpbin.org").await;
    /// }
    /// ```
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, HttpClientError> {
        self.config.limit.until_ready().await;
        self.config.client.get(url).send().await.map_err(HttpClientError::Request)
    }
    
    /// The Client performs a get request using a registered host. The host is extracted 
    /// from `url`.  You can register a host using the `build_host` method. 
    /// 
    /// If you use `get` instead of `host_get`, this request will use the 
    /// global quota instead of host's quota. No error is triggered. 
    /// 
    /// # Errors
    /// - If the host doesn't exist (It isn't registered), a 
    /// `HostNotFound` will be returned.
    /// - If the `url` is in a invalid format, a `Parse` error is returned. 
    /// - If the `url` doesn't contain any *hostname*, a `NoHostname` is returned. 
    /// 
    /// Any other errors return a `Request` error.
    /// # Example
    /// Sets a config and client, then register a host.  The host contains the same 
    /// config than *global*, but it can be changed specifing a `HostConfig` with a 
    /// different `base` field.
    /// ```
    /// # use rate_limit_client::{RateLimitClient, TimeInterval};
    /// # use rate_limit_client::configs::{Config, HostConfig};
    /// # use std::num::NonZeroU32;
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = Config {
    ///        quota: NonZeroU32::new(10).unwrap(),
    ///        burst: NonZeroU32::new(2).unwrap(),
    ///        interval: TimeInterval::BySeconds,  
    ///     };
    ///     let client = RateLimitClient::build(config);
    ///     
    ///     client.build_host(HostConfig {
    ///         base: config,
    ///         hostname: "httpbin"
    ///     });
    ///     
    ///     client.host_get("https://httpbin.org").await;
    /// }
    /// ```
    pub async fn host_get(&self, url: &str) -> Result<reqwest::Response, HttpClientError> {
        let host = get_host(url)?;
        
        if let Some(host) = self.hosts.get(&host) {
            host.quota.until_ready().await;
            self.config.client.get(url).send().await.map_err(HttpClientError::Request)
        } else {
            Err(HttpClientError::HostNotFound(host))
        }
    }
}

impl<C> RateLimitClient<C>
where
    C: Clock + Clone,
    C::Instant: Reference,
{
    #[doc(hidden)]
    /// Builds a client with a clock. It's common in testing scenarios. It's strongly recommended
    /// to build a client using `build`.
    pub fn build_with_clock(config: ConfigWithClock<C>) -> Self {
        let config = GlobalConfig::build_with_clock(config);

        Self {
            config,
            hosts: DashMap::new(),
        }
    }
    
    #[doc(hidden)]
    /// Check is there are some ticket available
    pub fn global_limit_is_ok(&self) -> bool {
        self.config.limit.check().is_ok()
    }
    #[doc(hidden)]
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
    /// use rate_limit_client::{
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
    pub fn build_host(&self, config: HostConfig) {
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
