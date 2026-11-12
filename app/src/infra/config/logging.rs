use serde::Deserialize;
use std::fmt;

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// 控制台日志格式（文件日志始终为 JSON）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleFormat {
    /// 多行、带颜色，适合开发
    Pretty,
    /// 单行，适合生产
    Compact,
}

/// 日志文件轮转方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogRotation {
    Daily,
    Hourly,
    Never,
}

/// 日志配置（`[logging]`）
#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别（可被 `RUST_LOG` 环境变量覆盖，以支持按模块过滤）
    pub level: LogLevel,

    /// 是否输出到控制台
    pub console: bool,

    /// 控制台日志格式
    pub console_format: ConsoleFormat,

    /// 是否输出到文件（JSON 格式）
    pub file: bool,

    /// 日志文件目录
    pub file_dir: String,

    /// 日志文件名前缀，最终文件名形如 `{file_prefix}.2026-01-01.log`
    pub file_prefix: String,

    /// 日志文件轮转方式
    pub rotation: LogRotation,

    /// 最多保留的日志文件数量，轮转时自动删除最旧的文件（0 表示不限制）
    pub max_files: usize,
}

impl LoggingConfig {
    pub(super) fn validate(&self) -> Result<(), String> {
        if !self.console && !self.file {
            return Err("console 和 file 至少需要启用一个".to_string());
        }
        if self.file && (self.file_dir.is_empty() || self.file_prefix.is_empty()) {
            return Err("启用文件日志时 file_dir 和 file_prefix 不能为空".to_string());
        }
        Ok(())
    }
}
