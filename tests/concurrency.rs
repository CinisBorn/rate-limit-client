use rate_limit_client::configs::{Config, HostConfig};
use rate_limit_client::errors::HttpClientError;
use rate_limit_client::{RateLimitClient, TimeInterval};
use std::num::NonZeroU32;
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