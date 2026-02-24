use reqwest::Client;
use std::time::Duration;
use tokio::task;

pub async fn fetch(endpoint: String, _freq: i32) {
    let client = Client::new();
    // TODO: implement a custum interval from user input
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;
        let mut handlers = Vec::new();

        let client = client.clone();
        let endpoint = endpoint.clone();

        handlers.push(task::spawn(async move {
            let response = client.get(endpoint).send().await.unwrap();
            println!("{:?}", response.status());
        }));

        for handle in handlers {
            handle.await.unwrap();
        }
    }
}
