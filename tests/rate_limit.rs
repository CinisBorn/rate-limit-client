use governor::clock::FakeRelativeClock;
use governor::{Quota, RateLimiter};
use std::num:: NonZeroU32;
use std::time::Duration;
use http_client::{RateLimitClient, TimeInterval};

#[test]
fn should_respect_limit() {
    let fake_clock = FakeRelativeClock::default();
    let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
    let limit = RateLimiter::direct_with_clock(quota, fake_clock);

    assert!(limit.check().is_ok());
    assert!(limit.check().is_err());
}

#[test]
fn shoud_respect_limit_in_request() {
    let clock = FakeRelativeClock::default();
    let quota = NonZeroU32::new(1).unwrap();
    let interval = http_client::TimeInterval::ByHours;
    let client = RateLimitClient::build_with_clock(quota, interval, clock.clone());

    let key = format!("https://supercalm.com");

    assert!(client.global_limit_is_ok(&key));
    assert!(client.global_limit_is_err(&key));

    clock.advance(Duration::from_secs(1));

    assert!(client.global_limit_is_err(&key));

    clock.advance(Duration::from_hours(1));

    assert!(client.global_limit_is_ok(&key));
    assert!(client.global_limit_is_err(&key));
}

#[test]
fn should_respect_limit_by_host() {
    let clock = FakeRelativeClock::default();
    let quota = NonZeroU32::new(1).unwrap();

    let host1 = "veryhappywithit.com";
    let host2 = "coolhost.com";
    
    let endpoint1 = "http://veryhappywithit.com";
    let endpoint2 = "http://coolhost.com";

    let host1_quota = NonZeroU32::new(10).unwrap();
    let host2_quota = NonZeroU32::new(5).unwrap();

    let global_interval = http_client::TimeInterval::ByHours;
    let host1_interval = http_client::TimeInterval::ByHours;
    let host2_interval = http_client::TimeInterval::ByMinutes;

    let mut client = RateLimitClient::build_with_clock(quota, global_interval, clock.clone());
    
    client.build_host(host1, host1_quota, host1_interval);
    client.build_host(host2, host2_quota, host2_interval);

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
    let time = TimeInterval::ByMinutes;
    let quota = NonZeroU32::new(10).expect("to work");
    let host = "httpbin.org";
    let url = "https://httpbin.org/get";
    let mut client = RateLimitClient::build_with_clock(quota, time, clock.clone());
    
    client.build_host(host, quota, TimeInterval::ByHours);
    
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
        
    let mut client = RateLimitClient::build(quota, TimeInterval::ByHours);

    client.build_host(host1, quota, TimeInterval::ByHours);
    client.build_host(host2, quota, TimeInterval::ByMinutes);
    
    assert!(client.host_exists(host1));
    assert!(client.host_exists(host2))
}