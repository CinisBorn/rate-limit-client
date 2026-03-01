use http_client::RateLimitClient;

use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn response_with_ok() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = RateLimitClient::build_default();
    let url = format!("{}/hello", &server.uri());

    let response = client.get(&url).await.unwrap();

    assert_eq!(response.status(), 200);
}
