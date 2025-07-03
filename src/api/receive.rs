use crate::BodyConfig;
use crate::utils::config::AppConfig;
use crate::utils::request;
use reqwest::Method;
use reqwest::header::HeaderMap;
use serde_json::json;

#[derive(Clone, Debug)]
pub struct Receive {
    pub header: HeaderMap,
    pub body: BodyConfig,
}

// 获取优惠券接口
pub async fn receive(params: Receive) {
    // 读取setting.ini 的并发数
    let config = AppConfig::from_ini("setting.ini");
    let request = request::Request::new(config.thread);
    let request_body = request::RequestBody {
        url: "https://mapi.uhaozu.com/api/community/coupon/center/receive".to_string(),
        method: Method::POST,
        headers: params.header,
        body: json!({
            "communityId": params.body.community_id,
            "redPackTaskId": params.body.red_pack_task_id,
            "zoneId": params.body.zone_id,
        }).to_string(),
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
                community_id: "54".to_string(),
                red_pack_task_id: "66578".to_string(),
                zone_id: "214".to_string(),
            },
        };
        receive(params).await;
    }
}
