use std::num::NonZeroU32;
use std::time::Duration;
use governor::clock::{FakeRelativeClock};
use governor::{Quota, RateLimiter};

use http_client::RateLimitClient;

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
    let client = RateLimitClient::build_with_clock(
        clock.clone(), 
        quota, 
        interval
    );
    
    let url = format!("https://supercalm.com");
    
    assert!(client.get_default_limit().check_key(&url).is_ok());
    assert!(client.get_default_limit().check_key(&url).is_err());
    
    clock.advance(Duration::from_secs(1)); 
    
    assert!(client.get_default_limit().check_key(&url).is_err());
    
    clock.advance(Duration::from_hours(1)); 
    
    assert!(client.get_default_limit().check_key(&url).is_ok());
    assert!(client.get_default_limit().check_key(&url).is_err());
}