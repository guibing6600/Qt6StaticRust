//! 全局配置（单一数据源）

/// 应用配置
pub struct Config {
    pub app_name: &'static str,
    /// 版本号跟随 Cargo.toml
    pub version: &'static str,
    /// 是否启用单实例防护：true 时已有实例则激活其窗口并退出
    pub single_instance: bool,
    /// 主窗口默认尺寸（启动时使用，窗口大小可自由调整）
    pub window_width: i32,
    pub window_height: i32,
}

pub const CONFIG: Config = Config {
    app_name: "Qt6StaticRust",
    version: env!("CARGO_PKG_VERSION"),
    single_instance: true,
    window_width: 720,
    window_height: 480,
};
