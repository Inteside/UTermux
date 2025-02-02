use crate::utils::request::{request, Headers};
use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
struct PhoneData {
    auth_token: String,
}

#[derive(Deserialize, Debug)]
struct PhoneObject {
    #[serde(rename = "mobilePhone")]
    mobile_phone: String,
}

#[derive(Deserialize, Debug)]
struct PhoneResponse {
    success: bool,
    object: PhoneObject,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenRecord {
    pub auth_token: String,
    pub mobile_phone: String,
    pub user_agent: String,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenStorage {
    pub records: Vec<TokenRecord>,
}

// 
pub fn get_config_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let app_config_dir = config_dir.join("UTermux");
    Some(app_config_dir.join("auth_token.json"))
}

// 保存token
fn save_token(token: &str, mobile_phone: &str, user_agent: &str) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(token_path) = get_config_path() {
        // 确保配置目录存在
        if let Some(parent) = token_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 读取或创建存储对象
        let mut storage = if token_path.exists() {
            let content = fs::read_to_string(&token_path)?;
            serde_json::from_str(&content).unwrap_or(TokenStorage { records: vec![] })
        } else {
            TokenStorage { records: vec![] }
        };

        // 检查是否存在相同手机号
        if storage.records.iter().any(|r| r.mobile_phone == mobile_phone) {
            return Ok(());
        }

        // 添加新记录
        storage.records.push(TokenRecord {
            auth_token: token.to_string(),
            mobile_phone: mobile_phone.to_string(),
            user_agent: user_agent.to_string(),
            active: storage.records.is_empty(), // 第一个记录设置为active
        });

        // 将对象序列化为JSON并保存
        let json_content = serde_json::to_string_pretty(&storage)?;
        fs::write(token_path, json_content)?;
    }
    Ok(())
}

// 修改read_saved_token函数以适应新的JSON格式
pub fn read_saved_token() -> Option<String> {
    get_config_path().and_then(|path| {
        fs::read_to_string(path).ok().and_then(|content| {
            let storage: TokenStorage = serde_json::from_str(&content).ok()?;
            storage
                .records
                .iter()
                .find(|r| r.active)
                .map(|r| r.auth_token.clone())
        })
    })
}

// 添加新函数来获取当前激活账号的user_agent
pub fn read_saved_user_agent() -> Option<String> {
    get_config_path().and_then(|path| {
        fs::read_to_string(path).ok().and_then(|content| {
            let storage: TokenStorage = serde_json::from_str(&content).ok()?;
            storage
                .records
                .iter()
                .find(|r| r.active)
                .map(|r| r.user_agent.clone())
        })
    })
}

// 查询手机号验证是否登录
pub async fn query_mobile_phone(
    auth_token: &str,
    user_agent: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let headers = Headers {
        auth_token: auth_token.to_string(),
        user_agent: Some(user_agent.to_string()),
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
                // 保存 token 到配置目录，现在也保存 user_agent
                save_token(auth_token, &json.object.mobile_phone, user_agent)?;

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
    let auth_token = "d8AZUpWrOsfV1GUfqhPS4EQ08BfnRuJ2xIRU0hrPRCD9l32AHpr5QgqtPysy5y_cLJ5vuDm34Cwj2fltIbFO6HFfVzG85e551gygs4JDeFOqUsScTsPNhF89U7XCb7Tp6UWvv2SAq22V2NfQW17DZUC8MNXD-zmIXV2AhZaBBNibJWcFfOD8wZQOb0oUyLzJwtOtW-owPLIeFdpuOC4w";
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 Edg/132.0.0.0";
    let response = query_mobile_phone(&auth_token, &user_agent).await;
    println!("{:?}", response.unwrap());
}
