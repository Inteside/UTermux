use crate::api::ApiResponse;
use crate::utils::request::{self, Data, Headers};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::{Duration, Instant};

// 添加静态变量来追踪 zoneId
static ZONE_ID_COUNTER: AtomicU32 = AtomicU32::new(1);

pub async fn fetch_receive(
    auth_token: String,
    red_pack_task_id: String,
    community_id: String,
) -> Result<String, ReceiveError> {
    let start_time = Instant::now();
    let timeout_duration = Duration::from_secs(5);
    let mut success = false;

    // 减少并发任务数量，但保持持续请求
    let mut handles = vec![];
    for _ in 0..5 {
        // 从10减少到5个并发任务
        let auth_token = auth_token.clone();
        let red_pack_task_id = red_pack_task_id.clone();
        let community_id = community_id.clone();

        let handle = tokio::spawn(async move {
            let mut last_error = None;

            while Instant::now().duration_since(start_time) < timeout_duration {
                let current_zone_id = ZONE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

                let data = Data {
                    communityId: community_id.clone(),
                    redPackTaskId: red_pack_task_id.clone(),
                    zoneId: current_zone_id.to_string(),
                };

                let headers = Headers {
                    auth_token: auth_token.clone(),
                    user_agent: crate::api::queryMobilePhone::read_saved_user_agent(),
                };

                match request::request(
                    PathBuf::from("community/coupon/center/receive"),
                    Some(data),
                    Some(headers),
                )
                .await
                {
                    Ok(response) => {
                        let json: ApiResponse = serde_json::from_str(&response).unwrap();
                        if json.success {
                            return Ok("领取成功".to_string());
                        } else if json.responseCode == "2040" {
                            return Err(ReceiveError(json.responseMsg)); // 已领取
                        }
                        last_error = Some(ReceiveError(json.responseMsg));
                    }
                    Err(e) => {
                        last_error = Some(ReceiveError(e.to_string()));
                    }
                }

                // 添加小延迟以减少CPU使用
                tokio::time::sleep(Duration::from_millis(20)).await;
            }

            Err(last_error.unwrap_or_else(|| ReceiveError("请求超时".to_string())))
        });

        // 在创建任务之间添加小延迟，避免瞬间创建大量任务
        tokio::time::sleep(Duration::from_millis(10)).await;
        handles.push(handle);
    }

    // 等待任何一个任务成功或所有任务完成
    let mut last_error = None;
    for handle in handles {
        match handle.await.unwrap() {
            Ok(msg) => {
                success = true;
                return Ok(msg);
            }
            Err(e) => {
                last_error = Some(e);
            }
        }
    }

    if success {
        Ok("领取成功".to_string())
    } else {
        Err(last_error.unwrap_or_else(|| ReceiveError("所有请求均失败".to_string())))
    }
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

#[tokio::test]
async fn test_fetch_info() {
    let response = fetch_receive("123".to_string(), "123".to_string(), "14".to_string()).await;
    println!("{:#?}", response);
}
