use std::num::NonZeroU32;

use http_client::{ClientBuilder, TimeMeasurement};
use wiremock::{
    Mock, 
    MockServer, 
    ResponseTemplate, 
    matchers::{path, method}
};

#[tokio::test]
async fn response_with_ok() {
    let server = MockServer::start().await;
    
    Mock::given(method("GET")) 
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    
    let quota = NonZeroU32::new(10).expect("No Zero");
    let interval = TimeMeasurement::ByMinutes;
    let client = ClientBuilder::build(quota, interval);
    let url = format!("{}/hello", &server.uri());
    
    let response = client.get(&url).await.unwrap();
    
    assert_eq!(response.status(), 200);
}
