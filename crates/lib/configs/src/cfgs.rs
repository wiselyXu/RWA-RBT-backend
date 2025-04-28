use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// 配置文件
#[derive(Debug, Deserialize)]
pub struct Configs {
    /// 程序配置
    pub server: Server,
    pub redis: Redis,
    pub jwt: Jwt,
    pub kafka: Kafka,
    ///  数据库 配置
    pub database: Database,

}

/// server 配置文件
#[derive(Clone,Debug, Deserialize)]
pub struct Server {
    /// 服务器名称
    pub name: String,
    /// 服务器名称
    pub version: String,
    /// 服务器名称
    pub debug: bool,
    /// 服务器(IP地址)
    pub ip: String,
    /// 服务器(端口)
    pub port: i32,
    /// 服务器名称
    pub api_prefix: String,
}

/// Redis 配置文件
#[derive(Clone,Debug, Deserialize)]
pub struct Redis {
    pub url: String,
}

#[derive(Clone,Debug, Deserialize)]
pub struct Jwt {
    pub secret: String,
}

/// Kafka 配置文件
#[derive(Debug, Deserialize)]
pub struct Kafka {
    pub url: String,
    pub producer_timeout_ms: u64,
    pub group: String,
    pub order_command_topic: String,
    pub order_match_topic: String,
    pub depth_topic: String,
    pub trade_topic: String,
    pub account_match_topic: String,
}

#[derive(Debug, Deserialize)]
pub struct Http {
    pub exchange_url: String,
    pub url: String,
}

/// 数据库
#[derive(Clone,Debug, Deserialize)]
pub struct Database {
    /// 数据库连接
    pub url: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 是否初始化数据库
    pub init_database: bool,
    /// 是否初始化数据表
    pub sync_tables: bool,
}

/// 数据库配置
#[derive(Debug, Deserialize)]
pub struct Tdengine {
    /// 服务器地址
    pub url: String,
    /// 数据库名称
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 是否启用
    pub enabled: bool,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "MayApp", about = "An example of StructOpt usage.")]
pub struct Opt {
    #[structopt(short = "e", default_value = "dev", parse(from_str))]
    pub env: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Nacos {
    pub server_addr: String,
    pub namespace: String,
    pub ip: String,
    pub exchange_service_name: String,
    pub market_service_name: String,
}
