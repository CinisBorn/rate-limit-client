use std::num::NonZeroU32;
use http_client::{RateLimitClient, TimeInterval};
use wiremock::{Mock, MockServer, ResponseTemplate};
use wiremock::matchers::{path, method};
use http_client::configs::Config;

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
        interval: TimeInterval::ByHours
    };
    
    let client = RateLimitClient::build(config);
    let response = client
        .get(&format!("{}/test", mock.uri()))
        .await
        .unwrap()
        .status();
    
    assert_eq!(response.as_u16(), 200)
}