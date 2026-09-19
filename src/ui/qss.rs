//! 全局样式表（QSS）集中管理
//!
//! - 所有样式集中在本文件；业务代码不写样式
//! - 应用方式：`ui::build_main_window` 里把 `global()` 挂到窗口根节点，
//!   规则级联到全部子控件

/// 全局样式
const TEMPLATE: &str = r#"
/* ---- 全局字体：全部界面统一 Consolas 14px ---- */
QWidget {
    font-family: 'Consolas';
    font-size: 14px;
}

QLabel {
    color: #57606a;
}
"#;

/// 生成全局样式表
pub fn global() -> String {
    TEMPLATE.to_string()
}
