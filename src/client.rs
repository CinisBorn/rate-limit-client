use reqwest::Client;

pub async fn fetch(endpoint: String, _freq: i32) {
    let client = Client::new();

    let cloned_client = client.clone();
    let cloned_endpoint = endpoint.clone();

    let _ = tokio::spawn(async move {
        let response = cloned_client
            .get(cloned_endpoint)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        println!("Request");
        println!("{:?}", response);
    }).await;
}
