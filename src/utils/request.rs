use base64;
use reqwest::header::HeaderValue;
use reqwest::{header, Client};
use serde::Serialize;
use serde_json;
use std::{path::PathBuf, vec};

#[derive(Serialize, Clone)]
pub struct Data {
    pub communityId: String,
    pub redPackTaskId: String,
    pub zoneId: String,
}

#[derive(Serialize)]
pub struct LoginData {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct Headers {
    pub auth_token: String,
    pub mobile_phone: String,
}

impl Headers {
    pub fn from_string(input: &str) -> Option<Self> {
        // 首先按换行符分割，然后过滤掉空行
        let parts: Vec<&str> = input.split('\n').filter(|s| !s.trim().is_empty()).collect();

        if parts.len() != 2 {
            return None;
        }

        // 使用 strip_prefix 并确保移除所有可能的空白字符
        let auth_token = parts[0].strip_prefix("AuthToken:")?.trim().to_string();

        let mobile_phone = parts[1].strip_prefix("MobilePhone:")?.trim().to_string();

        // 打印 auth_token 的内容和长度
        println!("Auth Token: {}", auth_token);
        println!("Auth Token length: {}", auth_token.len());
        // 打印每个字符的 ASCII 值，以检查是否有不可见字符
        println!("Auth Token bytes: {:?}", auth_token.as_bytes());

        Some(Headers {
            auth_token,
            mobile_phone,
        })
    }
}

// 请求函数
pub async fn request(
    path: PathBuf,
    data: Option<impl serde::Serialize>,
    headers: Option<Headers>,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("https://mapi.uhaozu.com/api/{}", path.display());
    let mut request = client.post(&url)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 Safari/537.36 Edg/132.0.0.0");

    if let Some(headers) = headers {
        // 使用写死的 auth_token
        let auth_token = "d8AZUpWrOsfV1GUfqhPS4EQ08BfnRuJ2xIRU0hrPRCD9l32AHpr5QgqtPysy5y_cLJ5vuDm34Cwj2fltIbFO6HFfVzG85e551gyntoRAelmhV8acTsPNhF89U7XCb7Tp6UWvv2SAq22V2NfQW17DZUC8MNXD-zmIXV2AhZaBBImZIjUFL7z8xscOaUYcnu2cy9n4VbMxOORORNVsb305";
        request = request.header("authToken", auth_token);

        if !headers.mobile_phone.is_empty() {
            let phone_header = HeaderValue::from_str(&headers.mobile_phone)
                .map_err(|e| format!("Invalid mobile phone: {}", e))?;
            request = request.header("mobilePhone", phone_header);
        }
    }

    // 只在有data时添加请求体
    if let Some(data) = data {
        request = request.json(&data);
    }
    let response = request.send().await?;
    Ok(response.text().await?)
}

#[tokio::test]
async fn test_request() {
    let input = "AuthToken:d8AZUpWrOsfV1GUfqhPS4EQ08BfnRuJ2xIRU0hrPRCD9l32AHpr5QgqtPysy5y_cLJ5vuDm34Cwj2fltIbFO6HFfVzG85e551gKisodEf16uUcScTsPNhF89U7XCb7Tp6UWvv2SAq22V2NfQW17DZUC8MNXD-zmIXV2AhZaBBN3NcGdSLOj-ncVZO0YWyeGby9-qArsxP7cfQtg4OSw8\nMobilePhone:19370944673";

    if let Some(headers) = Headers::from_string(input) {
        println!("Auth Token: {}", headers.auth_token);
        println!("Mobile Phone: {}", headers.mobile_phone);

        let response = Client::new()
            .get("https://mapi.uhaozu.com/api/userBase/queryMobilePhone")
            .header("authToken", headers.auth_token)
            .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .send()
            .await;
        println!("{}", response.unwrap().text().await.unwrap());
    }
}
