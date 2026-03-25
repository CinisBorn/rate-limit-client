use governor::RateLimiter;
use governor::clock::Clock;
use governor::middleware::NoOpMiddleware;
use governor::state::{InMemoryState, NotKeyed};

type Middleware<C> = NoOpMiddleware<<C as Clock>::Instant>;
pub type DirectLimiter<C> = RateLimiter<NotKeyed, InMemoryState, C, Middleware<C>>;