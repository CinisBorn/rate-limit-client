use governor::Quota;
use std::num::NonZeroU32;
use crate::TimeInterval;

pub fn build_quota(quota: NonZeroU32, interval: TimeInterval) -> Quota {
    let burst = NonZeroU32::new(1).expect("Be the number 1");
    
    match interval {
        TimeInterval::ByHours => Quota::per_hour(quota).allow_burst(burst),
        TimeInterval::ByMinutes => Quota::per_minute(quota).allow_burst(burst),
        TimeInterval::BySeconds => Quota::per_second(quota).allow_burst(burst),
    }
}