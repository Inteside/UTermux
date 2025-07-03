use std::sync::Arc;
use reqwest::Client;
use reqwest::{header::HeaderMap, Method};
use tokio::sync::Semaphore;

pub struct Request {
    client: Client,
    semaphore: Arc<Semaphore>,
}

// 请求体
pub struct RequestBody {
    pub url: String,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: String,
}

impl Request {
    pub fn new(concurrency: usize) -> Self {
        Self {
            client: Client::new(),
            semaphore: Arc::new(Semaphore::new(concurrency)),
        }
    }

    pub async fn request(&self, request_body: RequestBody) -> Result<String, reqwest::Error> {
        let _permit = self.semaphore.acquire().await.unwrap();
        let response = self
            .client
            .request(request_body.method, &request_body.url)
            .headers(request_body.headers)
            .body(request_body.body)
            .send()
            .await?;
        Ok(response.text().await?)
    }
}
