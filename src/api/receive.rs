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
    println!("result: {:?}", result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_receive() {
        let mut header = HeaderMap::new();
        header.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36".parse().unwrap());
        header.insert("Authorization", "d8AZUpWrOsfV1GUfqhPS4EQ08BfnRuJ2xIRU0hrPRCD9l32AHpr5QgqtPysy5y_cLJ5vuDm34Cwj2fltIbFO6HFfVzG85e550Q2svY1FfVisWsScTsPNhF89U7XCb7Tp6UWvv2SAq22V2NfQW17DZUC8MNXD".parse().unwrap());
        let body = BodyConfig {
            communityId: "54".to_string(),
            redPackTaskId: "66578".to_string(),
            zoneId: "214".to_string(),
        };
        let params = Receive {
            header: header,
            body: body,
        };
        receive(params).await;
    }
}


