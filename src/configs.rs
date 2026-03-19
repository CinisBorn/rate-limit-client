use reqwest::Client;
use std::{num::NonZeroU32, ops::Deref};
use governor::{self, RateLimiter, clock::{Clock, DefaultClock, Reference}};

use crate::{TimeInterval, types::DirectLimiter};
use crate::build_quota;

/// A config `struct` for build clients and hosts.
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
/// A configuration `struct` for build `hosts`.
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
