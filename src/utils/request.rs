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
}

// 请求函数
pub async fn request<T>(
    path: PathBuf,
    data: Option<T>,
    headers: Option<Headers>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>>
where
    T: Serialize,
{
    let client = Client::new();
    let url = format!("https://mapi.uhaozu.com/api/{}", path.display());

    let mut request = client
        .post(&url)
        .header("authToken", headers.unwrap().auth_token)
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36");

    // 只在有data时添加请求体
    if let Some(data) = data {
        request = request.json(&data);
    }

    let response = request.send().await?;
    Ok(response.text().await?)
}

#[tokio::test]
async fn test_request() {
    let reponse = Client::new()
        .get("https://mapi.uhaozu.com/api/userBase/queryMobilePhone")
        .header("authToken", "d8AZUpWrOsfV1GUfqhPS4EQ08BfnRuJ2xIRU0hrPRCD9l32AHpr5QgqtPysy5y_cLJ5vuDm34Cwj2fltIbFO6HFfVzG85e551gKisodEf16uUcScTsPNhF89U7XCb7Tp6UWvv2SAq22V2NfQW17DZUC8MNXD-zmIXV2AhZaBBN3NcGdSLOj-ncVZO0YWyeGby9-qArsxP7cfQtg4OSw8")
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .send()
        .await;
    println!("{}", reponse.unwrap().text().await.unwrap());
}
