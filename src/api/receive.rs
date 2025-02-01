use crate::api::ApiResponse;
use crate::utils::request::{self, Data, Headers};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

// 添加静态变量来追踪 zoneId
static ZONE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

struct ReceiveData {
    communityId: String,
    redPackTaskId: String,
    zoneId: String,
}

#[derive(Debug)]
pub struct ReceiveError(pub String);

unsafe impl Send for ReceiveError {}
unsafe impl Sync for ReceiveError {}

impl std::error::Error for ReceiveError {}
impl std::fmt::Display for ReceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub async fn fetch_receive(
    auth_token: String,
    red_pack_task_id: String,
    community_id: String,
) -> Result<String, ReceiveError> {
    // 创建一个超时计时器
    let start_time = std::time::Instant::now();
    let timeout_duration = std::time::Duration::from_secs(5);
    loop {
        // 检查是否超时
        if start_time.elapsed() >= timeout_duration {
            return Err(ReceiveError("请求超时".to_string()));
        }

        // 获取当前计数并加1
        let current_zone_id = ZONE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        let data = Data {
            communityId: community_id.clone(),
            redPackTaskId: red_pack_task_id.clone(),
            zoneId: current_zone_id.to_string(),
        };

        let headers = Headers {
            auth_token: auth_token.clone(),
            mobile_phone: String::new(), // 如果不需要手机号，可以传空字符串
        };

        match request::request(
            PathBuf::from("community/coupon/center/receive"),
            Some(data),
            Some(headers),
        )
        .await
        {
            Ok(response) => {
                match serde_json::from_str::<ApiResponse>(&response) {
                    Ok(json) => {
                        if json.success {
                            return Ok("领取成功".to_string());
                        } else if json.responseCode == "2040" {
                            return Err(ReceiveError(json.responseMsg));
                        } else {
                            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                            continue;
                        }
                    },
                    Err(e) => {
                        println!("JSON解析错误: {}, 响应内容: {}", e, response);
                        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        continue;
                    }
                }
            }
            Err(e) => {
                // println!("请求错误:{:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                continue;
            }
        }
    }
}

#[tokio::test]
async fn test_fetch_info() {
    let response = fetch_receive("123".to_string(), "123".to_string(), "14".to_string()).await;
    println!("{:#?}", response);
}
