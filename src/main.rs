mod api;
mod utils;

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use api::receive;
use reqwest::header::HeaderMap;

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
#[derive(Serialize, Deserialize, Debug)]
pub struct HeaderConfig {
    pub auth_token: String,
    pub user_agent: String,
}

// Body
#[derive(Serialize, Deserialize, Debug)]
pub struct BodyConfig {
    pub communityId: String,
    pub redPackTaskId: String,
    pub zoneId: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config_content = fs::read_to_string(&cli.config)
        .expect("无法读取配置文件，请检查路径");
    println!("config_content: {}", config_content);
    let account: ConfigFile = serde_json::from_str(&config_content)
        .expect("配置文件格式错误");
    println!("账号配置: {:#?}", account);
    println!("定时: {} 秒, 并发: {}", cli.interval, cli.concurrency);

    // 设置请求头
    let mut header = HeaderMap::new();
    header.insert("User-Agent", account.header.user_agent.parse().unwrap());
    header.insert("authtoken", account.header.auth_token.parse().unwrap());
    header.insert("Content-Type", "application/json".parse().unwrap());

    // 设置请求参数
    let params = receive::Receive { header, body: account.body };
    receive::receive(params).await;
}
