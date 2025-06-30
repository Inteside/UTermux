use crate::utils::{request, request::Request, request::RequestBody};
use crate::{BodyConfig, HeaderConfig};
use reqwest::Method;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug)]
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
        // 打印请求之后的请求头
    } else {
        println!("请求成功: {:?}", result.unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    // 测试获取优惠券接口
    async fn test_other_url() {
        let params = Receive {
            header: HeaderMap::new(),
            body: BodyConfig {
                communityId: "54".to_string(),
                redPackTaskId: "66578".to_string(),
                zoneId: "214".to_string(),
            },
        };
        receive(params).await;
    }
}
