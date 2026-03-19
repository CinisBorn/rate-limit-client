//! Exports important long compound types necessary to abstract complex
//! governor types.
use governor::RateLimiter;
use governor::clock::Clock;
use governor::middleware::NoOpMiddleware;
use governor::state::{InMemoryState, NotKeyed};

type Middleware<C> = NoOpMiddleware<<C as Clock>::Instant>;

/// Type for creating hosts with separate quota.
pub type DirectLimiter<C> = RateLimiter<NotKeyed, InMemoryState, C, Middleware<C>>;