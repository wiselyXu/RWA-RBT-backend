#![allow(warnings)]
pub mod cfgs;
pub mod get_config;

// 重新导出
pub use get_config::CFG;

/// 配置错误类型
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// 配置文件不存在
    #[error("Config file not found: {0}")]
    FileNotFound(String),

    /// 配置解析错误
    #[error("Failed to parse config: {0}")]
    ParseError(String),

    /// 配置验证错误
    #[error("Config validation failed: {0}")]
    ValidationError(String),
}
