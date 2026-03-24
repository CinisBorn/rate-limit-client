# Description 

This project is a rate-limited HTTP client. It limits the number of requests 
made based on a configured quota and time interval.

## Example:

Let's suppose we want to get information from `api.github.com` without authentication. It has a limit of 60 requests per hour. We want to prevent 429 (Too Many Requests) errors

```rust
use rate_limit_client::{
    RateLimitClient, 
    TimeInterval,
    configs::{
        Config, 
        HostConfig
    }
};
use std::{error::Error, num::NonZeroU32};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = RateLimitClient::build(Config {
        quota: NonZeroU32::new(10).unwrap(),
        burst: NonZeroU32::new(2).unwrap(),
        interval: TimeInterval::ByMinutes,
    });
    
    client.build_host(HostConfig {
        base: Config {
            quota: NonZeroU32::new(60).unwrap(),
            burst: NonZeroU32::new(1).unwrap(),
            interval: TimeInterval::ByHours,
        },
        hostname: "api.github.com",
    });
    
    for i in 1..=100 {
        let user = format!("https://api.github.com/users/user{}", i);
        let response = client.host_get(&user).await.unwrap();
        
        println!("{}", response.status());
    }
    
    Ok(())
}
```
It will perform up to 10 requests per minute, distributed evenly (one request every 6 seconds).

## Configurations 

In the example above, we have this code snippet: 
```rust
    let client = RateLimitClient::build(Config {
        quota: NonZeroU32::new(10).unwrap(),
        burst: NonZeroU32::new(2).unwrap(),
        interval: TimeInterval::ByMinutes,
    });
```
It internally calculates the rate as quota/interval. In this case: 10 requests per 60 seconds = 1 request every 6 seconds.

The burst is the maximum number of tokens that the client can accumulate during an idle period. In this case, after 12 seconds of idle time, two tokens accumulate. When you make a request after this idle period, you can make two simultaneous requests immediately.

## Host Configuration:

In the example shown above, we configure a specific host with this code:
```rust
    client.build_host(HostConfig {
        base: Config {
            quota: NonZeroU32::new(60).unwrap(),
            burst: NonZeroU32::new(1).unwrap(),
            interval: TimeInterval::ByHours,
        },
        hostname: "api.github.com",
    });
```
This configures rate limiting specifically for the host `api.github.com`.

## Global Quota vs Host Quota

The main difference is that the global quota is shared among all non-registered hosts. In the following example, requests are made with the `get` method.

```rust
use rate_limit_client::{
    RateLimitClient, 
    TimeInterval,
    configs::{
        Config, 
        HostConfig
    }
};
use std::{error::Error, num::NonZeroU32};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = RateLimitClient::build(Config {
        quota: NonZeroU32::new(10).unwrap(),
        burst: NonZeroU32::new(2).unwrap(),
        interval: TimeInterval::ByMinutes,
    });
    
    for i in 1..=100 {
        let user = format!("https://api.github.com/users/user{}", i);
        let response = client.get(&user).await.unwrap();
        
        println!("{}", response.status());
    }
    
    Ok(())
}
```
This will make requests at a rate of one every 6 seconds (10 per minute). However, this will eventually fail because GitHub's API has a limit of 60 requests per hour, and you will receive HTTP 429 (Too Many Requests) errors.

You can wrap the client in an `Arc` to share it across multiple tasks (strongly recommended for concurrent usage):
```rust
let client = Arc::new(RateLimitClient::build(config));

for i in 1..=100 {
    let client = Arc::clone(&client);
    tokio::spawn(async move {
        client.get(&url).await
    });
}
```

All spawned tasks will share the same rate limiter, ensuring the total rate across all tasks respects the configured quota.

In contrast to the global quota, you can register a host with its own specific configuration. Once you have registered a host, you can make requests using the `host_get` method. This will use the host's specific quota instead of the global quota. If you use `get` instead of `host_get`, you will use the global quota, even for registered hosts.

# Roadmap

1. Refatore the code 
2. Add more HTTP methods (POST, PATCH, DELETE...)
3. Add more configuration options 

Next steps soon!