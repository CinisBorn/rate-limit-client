use rate_limit_client::configs::{Config, HostConfig, TimeInterval};
use rate_limit_client::errors::HttpClientError;
use rate_limit_client::RateLimitClient;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::time::Instant;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn should_get_200() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock)
        .await;

    let config = Config {
        quota: NonZeroU32::new(10).unwrap(),
        burst: NonZeroU32::new(10).unwrap(),
        interval: TimeInterval::ByHours,
    };

    let client = RateLimitClient::build(config);
    let response = client
        .get(&format!("{}/test", mock.uri()))
        .await
        .unwrap()
        .status();

    assert_eq!(response.as_u16(), 200)
}

#[tokio::test]
async fn host_should_get_successfully() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock)
        .await;

    let config = Config {
        quota: NonZeroU32::new(10).unwrap(),
        burst: NonZeroU32::new(10).unwrap(),
        interval: TimeInterval::ByHours,
    };
    
    let host_config = HostConfig {
        base: config,
        hostname: "127.0.0.1"
    };

    let client = RateLimitClient::build(config);
    
    client.build_host(host_config);
    
    let response = client
        .host_get(&format!("{}/test", mock.uri()))
        .await
        .unwrap()
        .status();

    assert_eq!(response.as_u16(), 200)
}

#[tokio::test]
async fn host_should_not_be_found() -> Result<(), ()>{
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock)
        .await;

    let config = Config {
        quota: NonZeroU32::new(10).unwrap(),
        burst: NonZeroU32::new(10).unwrap(),
        interval: TimeInterval::ByHours,
    };
    
    let host_config = HostConfig {
        base: config,
        hostname: "httpbin"
    };

    let client = RateLimitClient::build(config);
    
    client.build_host(host_config);
    
    let response = client
        .host_get(&format!("{}/test", mock.uri()))
        .await
        .unwrap_err();

    match response {
        HttpClientError::HostNotFound(_) => Ok(()),
        HttpClientError::NoHostname(_) => Err(()),
        HttpClientError::ParseHostError(_) => Err(()),
        HttpClientError::Request(_) => Err(())
    }
}

#[tokio::test]
async fn concurrent_requests_respect_limit() {
    let client = Arc::new(RateLimitClient::build(Config {
        quota: NonZeroU32::new(10).unwrap(),
        burst: NonZeroU32::new(1).unwrap(),
        interval: TimeInterval::BySeconds,
    }));
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for _ in 0..100 {
        let client = Arc::clone(&client);
        handles.push(tokio::spawn(async move {
            client.get("https://httpbin.org/get").await
        }));
    }
    
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
    
    let elapsed = start.elapsed();
    // 100 requests a 10/seg = ~10 segundos
    assert!(elapsed.as_secs() >= 9 && elapsed.as_secs() <= 11);
}