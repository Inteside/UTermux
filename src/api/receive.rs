use crate::api::ApiResponse;
use crate::utils::request::{self, Data, Headers};
use dirs::config_dir;
use std::fs;
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
    let timeout_duration = Duration::from_secs(2); // 控制请求超时时间

    // 减少并发数量到2个
    let mut handles = vec![];
    for _ in 0..1 {
        let auth_token = auth_token.clone();
        let red_pack_task_id = red_pack_task_id.clone();
        let community_id = community_id.clone();

        let handle = tokio::spawn(async move {
            let mut last_error = None;

            while Instant::now().duration_since(start_time) < timeout_duration {
                // let current_zone_id = ZONE_ID_COUNTER.fetch_add(2, Ordering::SeqCst);
                let current_zone_id = fs::read_to_string(format!(
                    "{}/UTermux/AppGame.json",
                    config_dir().unwrap().to_str().unwrap()
                ))
                .unwrap();

                let data = Data {
                    communityId: community_id.clone(),
                    redPackTaskId: red_pack_task_id.clone(),
                    zoneId: current_zone_id.to_string(),
                };

                let headers = Headers {
                    auth_token: auth_token.clone(),
                    user_agent: crate::api::queryMobilePhone::read_saved_user_agent(),
                };

                // 增加单个请求超时时间
                match tokio::time::timeout(
                    Duration::from_secs(2), // 增加到2秒
                    request::request(
                        PathBuf::from("community/coupon/center/receive"),
                        Some(data),
                        Some(headers),
                    ),
                )
                .await
                {
                    Ok(Ok(response)) => {
                        let json: ApiResponse = match serde_json::from_str(&response) {
                            Ok(json) => json,
                            Err(_) => {
                                last_error = Some(ReceiveError("响应格式错误".to_string()));
                                continue;
                            }
                        };

                        if json.success {
                            return Ok("领取成功".to_string());
                        } else if json.responseCode == "2040" {
                            return Err(ReceiveError(json.responseMsg));
                        }
                        last_error = Some(ReceiveError(json.responseMsg));
                    }
                    Ok(Err(e)) => {
                        last_error = Some(ReceiveError(e.to_string()));
                    }
                    Err(_) => {
                        last_error = Some(ReceiveError("请求超时".to_string()));
                    }
                }

                // 增加请求间隔
                tokio::time::sleep(Duration::from_millis(100)).await;
            }

            Err(last_error.unwrap_or_else(|| ReceiveError("请求超时".to_string())))
        });

        handles.push(handle);

        // 增加任务创建间隔
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let mut last_error = None;
    for handle in handles {
        match handle.await.unwrap() {
            Ok(msg) => return Ok(msg),
            Err(e) => last_error = Some(e),
        }
    }

    Err(last_error.unwrap_or_else(|| ReceiveError("所有请求均失败".to_string())))
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

#[tokio::test]
async fn test_read_app_game_json() {
    let app_game_json = fs::read_to_string(format!(
        "{}/UTermux/AppGame.json",
        config_dir().unwrap().to_str().unwrap()
    ))
    .unwrap();
    println!("{}", app_game_json);
}
