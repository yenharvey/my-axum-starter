use app::infra::{logging, server};
use app::{AppConfig, AppError};

/// 应用程序主入口点
///
/// 只负责加载配置、初始化日志，然后把控制权交给 [`server::run`]。
#[tokio::main]
async fn main() -> Result<(), AppError> {
    let config = AppConfig::load()?;

    // guard 必须存活到进程退出，否则缓冲中的日志不会被写出
    let _log_guard = logging::init(&config.logging)?;

    server::run(config).await
}
