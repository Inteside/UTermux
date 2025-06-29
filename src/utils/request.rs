use reqwest::Client;
use reqwest::{Method, header::HeaderMap};
use std::path::PathBuf;

pub struct Request {
    client: Client,
}

// 请求体
pub struct RequestBody {
    pub url: String,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: String,
}

impl Request {
    pub fn new() -> Self {
        Self {
            client: Client::builder().danger_accept_invalid_certs(true).build().unwrap(),
        }
    }

    pub async fn request(&self, request_body: RequestBody) -> Result<String, reqwest::Error> {
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
