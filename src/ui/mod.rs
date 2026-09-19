//! 主窗口：代码直接构建的最小框架（无 .ui 文件）
//!
//! 扩展约定：业务控件加在 `content` 布局里；样式统一写在 `qss.rs`。

mod qss;

use qtrs::prelude::*;

use crate::config::CONFIG;

/// 构建主窗口并返回句柄（调用方保持存活直至事件循环退出）
pub fn build_main_window() -> Widget {
    let icon_path: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/logo/logo.png");

    let mut window = Widget::new()
        .title(CONFIG.app_name)
        .size(CONFIG.window_width, CONFIG.window_height)
        .icon(icon_path)
        .build();

    // 全局样式挂在窗口根节点，级联到全部子控件
    window.set_style_sheet(&qss::global());

    // 内容区：业务控件从这里加起
    let layout = VBoxLayout::with_parent(&window);
    window.set_layout(&layout);

    window
}
