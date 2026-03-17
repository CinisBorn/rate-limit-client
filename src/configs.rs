use std::{num::NonZeroU32, ops::Deref};
use crate::{TimeInterval};
use governor::clock::Clock;

/// A config `struct` for build clients and hosts.
/// ```rust
/// use http_client::configs::Config;
/// use http_client::{TimeInterval, RateLimitClient};
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
/// use http_client::configs::{Config, HostConfig};
/// use http_client::{TimeInterval, RateLimitClient};
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
pub struct ConfigWithClock<C: Clock + Clone>{
    pub base: Config,
    pub clock: C,
}

impl<C: Clock + Clone> Deref for ConfigWithClock<C> {
    type Target = Config;
    
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}