/// Time interval unit for rate limiting calculations.
///
/// Specifies the time window over which the quota is distributed. Used in
/// conjunction with the `quota` field in [`Config`] to determine the rate.
///
/// # Calculation
///
/// The effective rate is calculated as `quota / interval`. For example:
/// - `quota: 60, interval: ByMinutes` = 60 requests per minute = 1 request/second
/// - `quota: 10, interval: BySeconds` = 10 requests per second
///
/// # Examples
///
/// ```
/// use rate_limit_client::{TimeInterval, RateLimitClient};
/// use rate_limit_client::configs::Config;
/// use std::num::NonZeroU32;
///
/// // 100 requests per hour
/// let client = RateLimitClient::build(Config {
///     quota: NonZeroU32::new(100).unwrap(),
///     burst: NonZeroU32::new(1).unwrap(),
///     interval: TimeInterval::ByHours,
/// });
/// ```
///
/// [`Config`]: crate::configs::Config

pub enum TimeInterval {
    /// Time interval in seconds.
    ///
    /// Use for high-frequency rate limits (e.g., 100 requests per second).
    BySeconds,
    
    /// Time interval in minutes.
    ///
    /// Use for moderate rate limits (e.g., 60 requests per minute).
    ByMinutes,
    
    /// Time interval in hours.
    ///
    /// Use for low-frequency rate limits (e.g., 1000 requests per hour).
    ByHours,
}

impl Copy for TimeInterval {}

impl Clone for TimeInterval {
    fn clone(&self) -> Self {
        *self
    }
}