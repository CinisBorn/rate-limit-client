//! This module provides all available configuration structs. 
//! 
//! `Config` is the *base* configuration for every other configuration type.
//! `HostConfig` and other types of configuration are derived from `Config`.

use reqwest::Client;
use std::{num::NonZeroU32, ops::Deref};
use governor::{self, RateLimiter, clock::{Clock, DefaultClock, Reference}};

use crate::{TimeInterval, types::DirectLimiter};
use crate::build_quota;

/// A base config struct for building clients and hosts.
/// 
/// The `quota` is the number of "tokens" allowed in a `interval`. The calculation is
/// `quota/interval` where `interval` is expressed in seconds. The result is the *frequency* 
/// at which *tokens* are recovered. 
/// 
/// The `burst` is the amount of *tokens* that can stack. If set to 2, the client 
/// can perform 2 requests at the start. When idle for `frequency * 2`, the stack is full 
/// again.
/// ```rust
/// use rate_limit_client::configs::Config;
/// use rate_limit_client::{TimeInterval, RateLimitClient};
/// use std::num::NonZeroU32;
///
/// fn main() {
///     let client = RateLimitClient::build(Config {
///         quota: NonZeroU32::new(10).unwrap(),
///         burst: NonZeroU32::new(2).unwrap(),
///         interval: TimeInterval::ByMinutes,
///     });
/// }
/// ```
pub struct Config {
    pub quota: NonZeroU32,
    pub burst: NonZeroU32,
    pub interval: TimeInterval,
}

impl Copy for Config {}
impl Clone for Config {
    fn clone(&self) -> Self {
        *self
    }
}
/// Sets the configuration for a specific host  
/// ```rust
/// use rate_limit_client::configs::{Config, HostConfig};
/// use rate_limit_client::{TimeInterval, RateLimitClient};
/// use std::num::NonZeroU32;
///
/// fn main() {
///     let config = Config {
///         quota: NonZeroU32::new(10).unwrap(),
///         burst: NonZeroU32::new(2).unwrap(),
///         interval: TimeInterval::ByMinutes,
///     };
///     let client = RateLimitClient::build(config);
///     
///     client.build_host(HostConfig {
///         base: config,
///         hostname: "google.com"
///     });
///     client.build_host(HostConfig {
///         base: Config {
///             quota: NonZeroU32::new(12).unwrap(),
///             burst: NonZeroU32::new(1).unwrap(),
///             interval: TimeInterval::ByMinutes,
///         },
///         hostname: "localhost.com"
///     });
/// }
/// ```
pub struct HostConfig {
    pub base: Config,
    pub hostname: &'static str,
}

impl Deref for HostConfig {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl Copy for HostConfig {}
impl Clone for HostConfig {
    fn clone(&self) -> Self {
        *self
    }
}
/// Creates a configuration to clients for testing purposes.
/// ```ignore
/// let config = RateLimitClient::build_with_clock(ConfigWithClock {
///     base: configs,
///     clock: FakeRelativeClock::default()
/// });
/// ```
#[doc(hidden)]
pub struct ConfigWithClock<C: Clock + Clone> {
    pub base: Config,
    pub clock: C,
}

impl<C: Clock + Clone> Deref for ConfigWithClock<C> {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[doc(hidden)]
/// Creates a global config to every client. Hosts and non-specified clients share this
/// configuration. It's not recommended to change it if you **do not know** what you are doing.
///
/// In this moment, there is no way to customize it, but in the future I plan add it.
#[derive(Debug)]
pub struct GlobalConfig<C: Clock + Clone> {
    pub limit: DirectLimiter<C>,
    pub client: Client,
    pub clock: C,
}

impl GlobalConfig<DefaultClock> {
    pub fn build(config: Config) -> Self {
        let internal_config = ConfigWithClock {
            base: config,
            clock: DefaultClock::default(),
        };

        GlobalConfig::build_with_clock(internal_config)
    }
}

impl<C> GlobalConfig<C>
where
    C: Clock + Clone,
    C::Instant: Reference,
{
    pub fn build_with_clock(config: ConfigWithClock<C>) -> Self {
        let limit = RateLimiter::direct_with_clock(
            build_quota(config.quota, config.burst, config.interval),
            config.clock.clone(),
        );

        Self {
            limit,
            client: Client::new(),
            clock: config.clock,
        }
    }
}
