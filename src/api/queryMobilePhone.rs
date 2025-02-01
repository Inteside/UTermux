use crate::utils::request::{request, Headers};
use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
struct PhoneData {
    auth_token: String,
}

#[derive(Deserialize)]
struct PhoneObject {
    mobile_phone: String,
}

#[derive(Deserialize)]
struct PhoneResponse {
    success: bool,
    object: PhoneObject,
}

pub fn get_config_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let app_config_dir = config_dir.join("UTermux");
    Some(app_config_dir.join("auth_token"))
}

// 保存token
fn save_token(token: &str, mobile_phone: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(token_path) = get_config_path() {
        // 确保配置目录存在
        if let Some(parent) = token_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 检查文件是否存在
        let content = if token_path.exists() {
            // 如果文件存在，读取现有内容
            fs::read_to_string(&token_path)?
        } else {
            String::new()
        };

        // 追加新的内容
        let new_content = format!(
            "{}AuthToken:{}\nMobilePhone:{}\n",
            content, token, mobile_phone
        );

        // 写入文件
        fs::write(token_path, new_content)?;
    }
    Ok(())
}

// 读取保存的token
pub fn read_saved_token() -> Option<String> {
    get_config_path().and_then(|path| fs::read_to_string(path).ok())
}

// 查询手机号验证是否登录
pub async fn query_mobile_phone(auth_token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let headers = Headers {
        auth_token: auth_token.to_string(),
        mobile_phone: String::new(),
    };
    let response = request(
        PathBuf::from("userBase/queryMobilePhone"),
        None::<PhoneData>,
        Some(headers),
    )
    .await
    .unwrap();

    // 如果不是json格式
    if !response.contains("success") {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "authToken输入错误或已过期".to_string(),
        )));
    } else {
        // 解析JSON响应
        let json: PhoneResponse = serde_json::from_str(&response)?;
        // 判断authToken能否登录
        match json.success {
            true => {
                // 保存 token 到配置目录
                save_token(auth_token, &json.object.mobile_phone)?;

                Ok(format!(
                    "{}:{}",
                    "登录成功!您的手机号是", json.object.mobile_phone
                ))
            }
            false => Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "authToken过期".to_string(),
            ))),
        }
    }
}

#[tokio::test]
async fn test_query_mobile_phone() {
    let auth_token = "d8AZUpWrOsfV1GUfqhPS4EQ08BfnRuJ2xIRU0hrPRCD9l32AHpr5QgqtPysy5y_cLJ5vuDm34Cwj2fltIbFO6HFfVzG85e551gKisodEf16uUcScTsPNhF89U7XCb7Tp6UWvv2SAq22V2NfQW17DZUC8MNXD-zmIXV2AhZaBBN3NcGdSLOj-ncVZO0YWyeGby9-qArsxP7cfQtg4OSw8";
    let response = query_mobile_phone(&auth_token).await;
    println!("{}", response.unwrap());
}
