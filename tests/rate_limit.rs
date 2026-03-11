use governor::clock::FakeRelativeClock;
use governor::{Quota, RateLimiter};
use std::num:: NonZeroU32;
use std::time::Duration;
use http_client::{RateLimitClient, TimeInterval};

// used to simplify the process of tests with common values. 
const DEFAULT_BURST: NonZeroU32 = NonZeroU32::new(1).unwrap();

#[test]
fn should_respect_limit() {
    let limit = RateLimiter::direct_with_clock(
        Quota::per_second(NonZeroU32::new(1).unwrap()), 
        FakeRelativeClock::default()
    );

    assert!(limit.check().is_ok());
    assert!(limit.check().is_err());
}

#[test]
fn shoud_respect_limit_in_request() {
    let clock = FakeRelativeClock::default();
    let client = RateLimitClient::build_with_clock(
        NonZeroU32::new(1).unwrap(), 
        DEFAULT_BURST, 
        TimeInterval::ByHours, 
        clock.clone()
    );

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

    let host1_interval = http_client::TimeInterval::ByHours;
    let host2_interval = http_client::TimeInterval::ByMinutes;
    
    let client = RateLimitClient::build_with_clock(
        NonZeroU32::new(1).unwrap(), 
        DEFAULT_BURST, 
        TimeInterval::ByHours, 
        clock.clone()
    );
    
    client.build_host(
        "veryhappywithit.com", 
        NonZeroU32::new(10).unwrap(), 
        DEFAULT_BURST, 
        host1_interval
    );
    client.build_host(
        "coolhost.com", 
        NonZeroU32::new(5).unwrap(), 
        DEFAULT_BURST, 
        host2_interval
    );

    assert!(client.host_limit_is_ok(endpoint1));
    assert!(client.host_limit_is_err(endpoint1));

    clock.advance(Duration::from_hours(1));

    assert!(client.host_limit_is_ok(endpoint1));
    assert!(client.host_limit_is_err(endpoint1));

    clock.advance(Duration::from_secs(1));

    assert!(client.host_limit_is_err(endpoint1));

    assert!(client.host_limit_is_ok(endpoint2));
    assert!(client.host_limit_is_err(endpoint2));

    clock.advance(Duration::from_mins(5));

    assert!(client.host_limit_is_ok(endpoint2));
    assert!(client.host_limit_is_err(endpoint2));
}

#[test]
fn should_use_correct_quota() {
    let clock = FakeRelativeClock::default();
    let url = "https://httpbin.org/get";
    let client = RateLimitClient::build_with_clock(
        NonZeroU32::new(10).unwrap(), 
        DEFAULT_BURST, 
        TimeInterval::ByMinutes, 
        clock.clone()
    );
    
    client.build_host(
        "httpbin.org", 
        NonZeroU32::new(10).unwrap(), 
        DEFAULT_BURST, 
        TimeInterval::ByHours
    );
    
    assert!(client.host_limit_is_ok(url));
    assert!(client.host_limit_is_err(url));
    
    clock.advance(Duration::from_mins(20));
    
    assert!(client.host_limit_is_ok(url));
}

#[test]
fn host_should_exists() {
    let quota = NonZeroU32::new(1).unwrap();
    let host1 = "veryhappywithit.com";
    let host2 = "coolhost.com";
    let client = RateLimitClient::build(
        quota, 
        DEFAULT_BURST, 
        TimeInterval::ByHours
    );

    client.build_host(host1, quota, DEFAULT_BURST, TimeInterval::ByHours);
    client.build_host(host2, quota, DEFAULT_BURST, TimeInterval::ByMinutes);
    
    assert!(client.host_exists(host1));
    assert!(client.host_exists(host2))
}