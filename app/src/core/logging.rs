use crate::error::AppError;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing(level: &str, format: &str) -> Result<(), AppError> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let subscriber = tracing_subscriber::registry().with(env_filter);

    match format {
        "json" => {
            subscriber
                .with(
                    tracing_subscriber::fmt::layer()
                        .json()
                        .with_file(true)
                        .with_line_number(true)
                        .with_target(false), // 不显示模块路径，减少噪音
                )
                .init();
        }
        "pretty" => {
            subscriber
                .with(
                    tracing_subscriber::fmt::layer()
                        .pretty()
                        .with_file(true)
                        .with_line_number(true)
                        .with_target(false),
                )
                .init();
        }
        _ => {
            // 默认使用 compact 格式 - 单行显示
            subscriber
                .with(
                    tracing_subscriber::fmt::layer()
                        .compact()
                        .with_file(true)
                        .with_line_number(true)
                        .with_target(false) // 不显示模块路径
                        .with_thread_ids(false) // 不显示线程ID
                        .with_thread_names(false), // 不显示线程名
                )
                .init();
        }
    }

    tracing::info!("Tracing 初始化完成，日志格式: {}", format);
    Ok(())
}
