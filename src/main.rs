use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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

#[derive(Serialize, Deserialize, Debug)]
struct AccountConfig {
    auth_token: String,
    user_agent: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config_content = fs::read_to_string(&cli.config)
        .expect("无法读取配置文件，请检查路径");
    let account: AccountConfig = serde_json::from_str(&config_content)
        .expect("配置文件格式错误");
    println!("账号配置: {:?}", account);
    println!("定时: {} 秒, 并发: {}", cli.interval, cli.concurrency);
    // TODO: 这里实现定时+并发抢票逻辑
}
