use crate::utils::{request::Request, request::RequestBody, request};
use std::path::PathBuf;
use reqwest::Method;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use crate::{HeaderConfig, BodyConfig};

pub struct Receive {
    pub header: HeaderMap,
    pub body: BodyConfig,
}

// 获取优惠券接口
pub async fn receive(params: Receive) {
    let request = request::Request::new();
    let request_body = request::RequestBody {
        url: "https://mapi.uhaozu.com/api/community/coupon/center/receive".to_string(),
        method: Method::POST,
        headers: params.header,
        body: serde_json::to_string(&params.body).unwrap(),
    };
    let result = request.request(request_body).await;
    if result.is_err() {
        println!("请求失败: {:?}", result.err());
    } else {
        println!("请求成功: {:?}", result.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    // 测试其他url
    async fn test_other_url() {
        let request = request::Request::new();
        let request_body = request::RequestBody {
            url: "https://www.google.com/".to_string(),
            method: Method::GET,
            headers: HeaderMap::new(),
            body: "".to_string(),
        };
        let result = request.request(request_body).await;
        if result.is_err() {
            println!("请求失败: {:?}", result.err());
        } else {
            println!("请求成功: {:?}", result.unwrap());
        }
    }
}


