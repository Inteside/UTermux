// 代理配置

pub struct ProxyConfig {
    pub proxy_list: Vec<String>,
}


impl ProxyConfig {
    pub fn new(proxy_list: Vec<String>) -> Self {
        Self { proxy_list }
    }

    pub async fn get_proxy(&self) -> Vec<String> {
        let response = reqwest::get("https://proxy.scdn.io/text.php");
        let body = response.await.unwrap();

        let proxy_list: Vec<String> = body
            .text()
            .await
            .unwrap()
            .split("\n")
            .map(|s| s.to_string())
            .collect();
        proxy_list
    }
}


#[tokio::test]
async fn test_get_proxy() {
    let proxy_config = ProxyConfig::new(vec![]);
    let proxy_list = proxy_config.get_proxy().await;
    println!("{:?}", proxy_list);
}

