//! 日志模块：tracing 写 exe 同目录的 running.log
//!
//! - 程序启动时最先调用 `logging::init()`
//! - 级别过滤默认 debug，可用 RUST_LOG 环境变量覆盖（如 RUST_LOG=info）
//! - 同步写入（无后台线程）：UI 日志量低频，每条即时落盘，退出无需 flush
//! - Release 构建依赖 tracing 的 release_max_level_off 特性，
//!   所有日志宏编译为空，零开销（本模块 init 为空实现）
//!
//! 调用方式：各模块直接用 `tracing::info! / warn! / error!` 等宏。

#[cfg(debug_assertions)]
pub fn init() {
    use tracing_subscriber::EnvFilter;

    let dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    // 每次启动清空上次日志
    let _ = std::fs::remove_file(dir.join("running.log"));

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(false)
        .with_writer(tracing_appender::rolling::never(&dir, "running.log"))
        // 本地时间（即中国时间），格式：2026-09-18 22:20:33.123
        .with_timer(tracing_subscriber::fmt::time::ChronoLocal::new(
            "%Y-%m-%d %H:%M:%S%.3f".to_string(),
        ))
        .init();

    tracing::info!("logging initialized (running.log)");
}

#[cfg(not(debug_assertions))]
pub fn init() {}
