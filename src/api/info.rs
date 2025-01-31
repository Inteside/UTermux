use crate::api::queryMobilePhone::read_saved_token;
use crate::utils::request::{self, Headers, LoginData};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoResponse {
    pub object: InfoObject,
    pub responseCode: String,
    pub responseMsg: String,
    pub success: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoData {
    object: InfoObject,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfoObject {
    pub zoneRedList: Vec<ZoneRed>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ZoneRed {
    redList: Vec<RedItem>,
    zoneName: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedItem {
    redPackTaskId: i64,
}

pub async fn get_info(
    auth_token: String,
    id: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let data = LoginData { id: id.to_string() };
    let headers = Some(Headers { auth_token });

    let response = request::request(
        PathBuf::from("community/coupon/center/info"),
        Some(data),
        headers,
    )
    .await
    .unwrap();

    let info: InfoResponse = serde_json::from_str(&response)?;

    // 收集所有的 redPackTaskId 和 zoneName
    let mut result = Vec::new();
    for zone in &info.object.zoneRedList {
        let mut ids = Vec::new();
        for red_item in &zone.redList {
            ids.push(red_item.redPackTaskId.to_string());
        }
        // 将zoneName和对应的ID列表组合
        result.push(format!("{}:{}", zone.zoneName, ids.join(",")));
    }

    Ok(result.join(";")) // 用分号分隔不同分类，分类名和ID之间用冒号分隔
}

#[tokio::test]
async fn test_get_info() {
    let auth_token = read_saved_token().unwrap();
    let red_pack_task_id = get_info(auth_token, "7".to_string()).await.unwrap();
    println!("RedPackTaskId: {}", red_pack_task_id);
}
