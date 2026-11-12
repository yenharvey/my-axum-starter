//! 日志系统初始化

use std::fs;

use tracing_appender::non_blocking::{self, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    EnvFilter, Layer, Registry, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::AppError;
use crate::infra::config::{ConsoleFormat, LogRotation, LoggingConfig};

type BoxedLayer = Box<dyn Layer<Registry> + Send + Sync>;

/// 日志后台写入线程的守卫
///
/// 必须持有到进程退出：被丢弃时会把缓冲中的日志全部写出。
#[must_use = "丢弃 LogGuard 会立即停止日志写入"]
pub struct LogGuard {
    _guards: Vec<WorkerGuard>,
}

/// 初始化 tracing 日志系统
///
/// - 控制台：`pretty` 或 `compact` 格式
/// - 文件：JSON 格式，按配置轮转并自动清理旧文件
///
/// 过滤级别取自 `logging.level`；如果设置了 `RUST_LOG` 环境变量则以它为准
/// （用于按模块过滤，如 `RUST_LOG=info,tower_http=debug`）。
pub fn init(config: &LoggingConfig) -> Result<LogGuard, AppError> {
    let mut layers: Vec<BoxedLayer> = Vec::new();
    let mut guards = Vec::new();

    if config.console {
        let (writer, guard) = non_blocking::NonBlocking::new(std::io::stdout());
        guards.push(guard);

        let layer = fmt::layer()
            .with_writer(writer)
            .with_ansi(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(false);
        layers.push(match config.console_format {
            ConsoleFormat::Pretty => layer.pretty().boxed(),
            ConsoleFormat::Compact => layer.compact().boxed(),
        });
    }

    if config.file {
        fs::create_dir_all(&config.file_dir)?;

        let (writer, guard) = non_blocking::NonBlocking::new(file_appender(config)?);
        guards.push(guard);

        layers.push(
            fmt::layer()
                .json()
                .with_writer(writer)
                .with_ansi(false)
                .with_file(true)
                .with_line_number(true)
                .with_target(false)
                .boxed(),
        );
    }

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(config.level.as_str()));

    tracing_subscriber::registry()
        .with(layers)
        .with(filter)
        .init();

    tracing::info!(
        level = %config.level,
        console = config.console,
        file = config.file,
        "日志系统初始化完成"
    );

    Ok(LogGuard { _guards: guards })
}

/// 创建按配置轮转的日志文件写入器
fn file_appender(config: &LoggingConfig) -> Result<RollingFileAppender, AppError> {
    let rotation = match config.rotation {
        LogRotation::Daily => Rotation::DAILY,
        LogRotation::Hourly => Rotation::HOURLY,
        LogRotation::Never => Rotation::NEVER,
    };

    let mut builder = RollingFileAppender::builder()
        .rotation(rotation)
        .filename_prefix(&config.file_prefix)
        .filename_suffix("log");
    if config.max_files > 0 {
        builder = builder.max_log_files(config.max_files);
    }

    builder
        .build(&config.file_dir)
        .map_err(|e| AppError::Io(std::io::Error::other(e)))
}
