use governor::clock::FakeRelativeClock;
use governor::{Quota, RateLimiter};
use rate_limit_client::configs::{Config, ConfigWithClock, HostConfig};
use rate_limit_client::{RateLimitClient, TimeInterval};
use std::num::NonZeroU32;
use std::time::Duration;

// used to simplify the process of tests with common values.
const DEFAULT_BURST: NonZeroU32 = NonZeroU32::new(1).unwrap();

#[test]
fn should_respect_limit() {
    let limit = RateLimiter::direct_with_clock(
        Quota::per_second(NonZeroU32::new(1).unwrap()),
        FakeRelativeClock::default(),
    );

    assert!(limit.check().is_ok());
    assert!(limit.check().is_err());
}

#[test]
fn shoud_respect_limit_in_request() {
    let clock = FakeRelativeClock::default();
    let config = ConfigWithClock {
        base: Config {
            quota: NonZeroU32::new(1).unwrap(),
            burst: DEFAULT_BURST,
            interval: TimeInterval::ByHours,
        },
        clock: clock.clone(),
    };
    let client = RateLimitClient::build_with_clock(config);
    let key = format!("https://supercalm.com");

    assert!(client.global_limit_is_ok(&key));
    assert!(client.global_limit_is_err(&key));

    clock.advance(Duration::from_secs(1));

    assert!(client.global_limit_is_err(&key));

    clock.advance(Duration::from_hours(1));

    assert!(client.global_limit_is_ok(&key));
}

#[test]
fn should_respect_limit_by_host() {
    let clock = FakeRelativeClock::default();

    let endpoint1 = "http://veryhappywithit.com";
    let endpoint2 = "http://coolhost.com";

    let host1_interval = rate_limit_client::TimeInterval::ByHours;
    let host2_interval = rate_limit_client::TimeInterval::ByMinutes;
    let config = ConfigWithClock {
        base: Config {
            quota: NonZeroU32::new(1).unwrap(),
            burst: DEFAULT_BURST,
            interval: TimeInterval::ByHours,
        },
        clock: clock.clone(),
    };

    let client = RateLimitClient::build_with_clock(config);

    client.build_host(HostConfig {
        base: Config {
            quota: NonZeroU32::new(10).unwrap(),
            burst: DEFAULT_BURST,
            interval: host1_interval,
        },
        hostname: "veryhappywithit.com",
    });
    client.build_host(HostConfig {
        base: Config {
            quota: NonZeroU32::new(5).unwrap(),
            burst: DEFAULT_BURST,
            interval: host2_interval,
        },
        hostname: "coolhost.com",
    });

    assert!(client.host_limit_is_ok(endpoint1));
    assert!(!client.host_limit_is_ok(endpoint1));

    clock.advance(Duration::from_hours(1));

    assert!(client.host_limit_is_ok(endpoint1));
    assert!(!client.host_limit_is_ok(endpoint1));

    clock.advance(Duration::from_secs(1));

    assert!(!client.host_limit_is_ok(endpoint1));

    assert!(client.host_limit_is_ok(endpoint2));
    assert!(!client.host_limit_is_ok(endpoint2));

    clock.advance(Duration::from_mins(5));

    assert!(client.host_limit_is_ok(endpoint2));
    assert!(!client.host_limit_is_ok(endpoint2));
}

#[test]
fn should_use_correct_quota() {
    let clock = FakeRelativeClock::default();
    let url = "https://httpbin.org/get";
    let config = ConfigWithClock {
        base: Config {
            quota: NonZeroU32::new(10).unwrap(),
            burst: DEFAULT_BURST,
            interval: TimeInterval::ByMinutes,
        },
        clock: clock.clone(),
    };
    let client = RateLimitClient::build_with_clock(config);

    client.build_host(HostConfig {
        base: Config {
            quota: NonZeroU32::new(10).unwrap(),
            burst: DEFAULT_BURST,
            interval: TimeInterval::ByHours,
        },
        hostname: "httpbin.org",
    });

    assert!(client.host_limit_is_ok(url));
    assert!(!client.host_limit_is_ok(url));

    clock.advance(Duration::from_mins(20));

    assert!(client.host_limit_is_ok(url));
}

#[test]
fn host_should_exists() {
    let quota = NonZeroU32::new(1).unwrap();
    let host1 = "veryhappywithit.com";
    let host2 = "coolhost.com";
    let config = Config {
        quota,
        burst: DEFAULT_BURST,
        interval: TimeInterval::ByHours,
    };
    let host_config1 = HostConfig {
        base: config,
        hostname: host1,
    };
    let mut host_config2 = host_config1;
    let client = RateLimitClient::build(config);

    host_config2.hostname = host2;

    client.build_host(host_config1);
    client.build_host(host_config2);

    assert!(client.host_exists(host1));
    assert!(client.host_exists(host2))
}
