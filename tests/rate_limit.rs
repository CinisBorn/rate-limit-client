use std::num::NonZeroU32;
use governor::clock::{FakeRelativeClock};
use governor::{Quota, RateLimiter};

#[test]
fn should_respect_limit() {
    let fake_clock = FakeRelativeClock::default();
    let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
    let limit = RateLimiter::direct_with_clock(quota, fake_clock);
    
    assert!(limit.check().is_ok());
    assert!(limit.check().is_err());
}