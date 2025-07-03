mod api;
mod utils;

use api::receive;
use chrono::{Duration as ChronoDuration, Local, NaiveTime};
use clap::Parser;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};
use tokio::time::{Duration, sleep};
use tokio::runtime::Builder;
use crate::utils::config::AppConfig;
use crate::utils::request::Request;
use futures::future::join_all;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 配置文件路径
    #[arg(short = 'f', long, default_value = "config.json")]
    config: PathBuf,

    /// 定时（秒）
    #[arg(short, long, default_value_t = 60)]
    interval: u64,

    /// 并发任务数
    #[arg(short = 'c', long, default_value_t = 1)]
    concurrency: usize,
}

// ConfigFile
#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigFile {
    pub header: HeaderConfig,
    pub body: BodyConfig,
}

// Header
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeaderConfig {
    pub auth_token: String,
    pub user_agent: String,
}

// Body
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BodyConfig {
    pub community_id: String,
    pub red_pack_task_id: String,
    pub zone_id: String,
}

fn main() {
    let config = AppConfig::from_ini("setting.ini");
    let runtime = Builder::new_multi_thread()
        .worker_threads(config.thread_num)
        .enable_all()
        .build()
        .unwrap();

    let request = Request::new(config.thread);
    runtime.block_on(async_main(request, config));
}

async fn async_main(_request: Request, config: AppConfig) {
    let cli = Cli::parse();
    let config_content = fs::read_to_string(&cli.config).expect("无法读取配置文件，请检查路径");
    println!("config_content: {}", config_content);
    let account: ConfigFile = serde_json::from_str(&config_content).expect("配置文件格式错误");
    println!("账号配置: {:#?}", account);
    println!("定时: {} 秒, 并发: {}", cli.interval, cli.concurrency);

    // 读取 setting.ini 的定时配置
    let mut trigger_time_str = String::from("18:59:59"); // 默认值
    if let Ok(file) = File::open("setting.ini") {
        let reader = BufReader::new(file);
        for line in reader.lines() {
            if let Ok(l) = line {
                if l.trim().starts_with("time=") {
                    if let Some(t) = l.trim().strip_prefix("time=") {
                        trigger_time_str = t.trim().to_string();
                    }
                }
            }
        }
    }
    println!("定时配置: {}", trigger_time_str);
    let target_time = NaiveTime::parse_from_str(&trigger_time_str, "%H:%M:%S")
        .unwrap_or_else(|_| NaiveTime::from_hms_opt(18, 59, 59).unwrap());

    // 设置请求头
    let mut header = HeaderMap::new();
    header.insert("User-Agent", account.header.user_agent.parse().unwrap());
    header.insert("authtoken", account.header.auth_token.parse().unwrap());
    header.insert("Content-Type", "application/json".parse().unwrap());

    // 设置请求参数
    let params = receive::Receive {
        header,
        body: account.body,
    };

    loop {
        // 获取当前本地时间
        let now = Local::now();
        // 构造今天的目标时间
        let today_target = now.date_naive().and_time(target_time);
        let next_trigger = if now.time() < target_time {
            today_target
        } else {
            // 已经过了今天的目标时间，等到明天
            (now.date_naive() + ChronoDuration::days(1)).and_time(target_time)
        };
        let duration_to_wait = next_trigger - now.naive_local();
        let secs = duration_to_wait.num_seconds();
        println!("距离下次请求还有 {} 秒 (目标时间: {})", secs, next_trigger);
        if secs > 0 {
            sleep(Duration::from_secs(secs as u64)).await;
        }
        // 到点后调用请求
        println!("到点，开始发送请求");
        // 按 send_num 并发调用 receive::receive
        let mut handles = vec![];
        for _ in 0..config.send_num {
            let params = params.clone();
            handles.push(tokio::spawn(async move {
                receive::receive(params).await;
            }));
        }
        join_all(handles).await;
    }
}

#[cfg(test)]
mod tests {
    use reqwest::{Client, Proxy};

    #[tokio::test]
    // 进行http代理请求测试
    async fn test_http_proxy() {
        // 创建代理配置
        let proxy = Proxy::http("http://103.41.81.176:80").unwrap();

        // 创建 HTTP 客户端并设置代理
        let client = Client::builder().proxy(proxy).build().unwrap();

        // 发送 GET 请求
        let response = client
            .get("http://api.ipify.org") // 使用这个 API 检查你的 IP
            .send()
            .await
            .unwrap();

        // 检查响应状态
        if response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap();
            // 如果响应状态码是404,则说明失败
            if status == 404 {
                println!("请求失败，状态码: {}", status);
                return;
            }
            println!("响应内容: {}", body);
        } else {
            println!("请求失败，状态码: {}", response.status());
            return;
        }
    }
}
