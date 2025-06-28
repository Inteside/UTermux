use reqwest::Client;
use reqwest::{header::HeaderMap, Method};
use std::path::PathBuf;

pub struct Request {
    client: Client,
}

// 请求体
pub struct RequestBody {
    pub path: PathBuf,
    pub method: Method,
    pub headers: HeaderMap,
    pub body: String,
}

impl Request {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn request(&self, request_body: RequestBody) -> Result<String, reqwest::Error> {
        let response = self
            .client
            .request(request_body.method, request_body.path.to_str().unwrap())
            .headers(request_body.headers)
            .body(request_body.body)
            .send()
            .await?;
        Ok(response.text().await?)
    }
}
