//! 启动框架：qtrs（Qt6 Widgets）+ 静态链接
//!
//! 流程：日志 → 单实例防护 → 静态插件注册 → 主窗口 → 事件循环。

mod config;
mod logging;
mod ui;

use qtrs::prelude::*;
use std::os::raw::c_void;

fn main() {
    // 日志最先初始化（写 exe 同目录 running.log，级别默认 debug，RUST_LOG 可覆盖）
    logging::init();
    tracing::info!("app starting ({} v{})", config::CONFIG.app_name, config::CONFIG.version);

    // 单实例防护（config::CONFIG.single_instance 可关闭）：
    // 已有实例时激活它的窗口然后退出
    let _instance_mutex = if config::CONFIG.single_instance {
        match acquire_single_instance_lock() {
            Ok(handle) => Some(handle),
            Err(()) => {
                tracing::warn!("another instance is running, activating it and exiting");
                activate_existing_window();
                return;
            }
        }
    } else {
        None
    };

    // 强制拉入静态插件注册对象并完成注册（见 src/cpp/plugin_registration.cpp）
    extern "C" {
        fn qt_register_static_plugins();
    }
    unsafe { qt_register_static_plugins() };
    tracing::info!("static plugins registered");

    let app = Application::new();
    tracing::info!("QApplication created");

    // 构建主窗口（居中显示）
    let window = ui::build_main_window();
    let (x, y) = centered_rect(config::CONFIG.window_width, config::CONFIG.window_height);
    window.move_to(x, y);
    window.show();
    tracing::info!("window shown, entering event loop");

    app.exec();
    tracing::info!("event loop exited");

    // 显式结束进程：qtrs 的 RAII 在复杂控件树上析构顺序存在隐患
    // （窗口/布局/控件交叉释放会卡死在退出阶段，产生残留进程锁住 exe）。
    // 跳过全部析构，由操作系统回收资源；日志为同步写入已全部落盘。
    tracing::info!("exiting");
    unsafe { ExitProcess(0) }
}

// --- user32/kernel32 最小 FFI：屏幕度量（居中）+ 单实例防护 ---

const SM_CXFULLSCREEN: i32 = 16; // 主屏工作区宽
const SM_CYFULLSCREEN: i32 = 17; // 主屏工作区高
const ERROR_ALREADY_EXISTS: i32 = 183;
const SW_RESTORE: i32 = 9;

extern "system" {
    fn GetSystemMetrics(index: i32) -> i32;
    fn ExitProcess(code: u32) -> !;
    fn CreateMutexW(attrs: *mut c_void, initial_owner: i32, name: *const u16) -> *mut c_void;
    fn FindWindowW(class: *const u16, title: *const u16) -> *mut c_void;
    fn IsIconic(hwnd: *mut c_void) -> i32;
    fn ShowWindow(hwnd: *mut c_void, cmd: i32) -> i32;
    fn SetForegroundWindow(hwnd: *mut c_void) -> i32;
}

/// 字符串转 UTF-16 null 结尾（Win32 宽字符 API 参数）
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// 单实例锁：创建命名互斥体，进程存活期间一直持有（句柄 drop 即释放）
/// 锁名由 config 的进程名拼接派生（Local\<进程名>SingleInstance），避免与其他应用冲突
/// 已存在实例时返回 Err
fn acquire_single_instance_lock() -> Result<*mut c_void, ()> {
    let name = wide(&format!(
        "Local\\{}SingleInstance",
        config::CONFIG.app_name
    ));
    let handle = unsafe { CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr()) };
    if handle.is_null() {
        // 创建失败按已存在处理，保守退出
        return Err(());
    }
    let err = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    if err == ERROR_ALREADY_EXISTS {
        return Err(());
    }
    Ok(handle)
}

/// 把已存在实例的窗口调到前台（最小化则先还原）
fn activate_existing_window() {
    let title = wide(config::CONFIG.app_name);
    let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
    if hwnd.is_null() {
        tracing::warn!("existing window not found by title");
        return;
    }
    unsafe {
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }
        SetForegroundWindow(hwnd);
    }
    tracing::info!("existing instance window activated");
}

/// 在主屏工作区（不含任务栏）居中放置 w×h 窗口的左上角坐标
fn centered_rect(w: i32, h: i32) -> (i32, i32) {
    unsafe {
        let cx = GetSystemMetrics(SM_CXFULLSCREEN);
        let cy = GetSystemMetrics(SM_CYFULLSCREEN);
        ((cx - w) / 2, (cy - h) / 2)
    }
}
