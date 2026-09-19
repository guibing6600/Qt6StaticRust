//! 构建脚本：
//! 1. exe 图标 + GUI 子系统（双击运行不弹控制台）
//! 2. 静态 Qt 链接补齐：qtrs 只链 Qt6Core/Gui/Widgets/UiTools 四个库，
//!    静态构建还需要 prl 文件里登记的依赖链（系统库/bundled 库/资源对象）
//!    以及 qwindows 平台插件的导入注册
use std::path::{Path, PathBuf};

/// 静态 Qt 根目录：优先读取 QT_PREFIX 环境变量，未设置时回退到默认安装位置
fn qt_prefix() -> String {
    std::env::var("QT_PREFIX")
        .unwrap_or_else(|_| r"D:\qt-everywhere-src-6.11.0\static".to_string())
}

fn main() {
    embed_app_icon();
    set_gui_subsystem();
    link_static_qt();
    compile_plugin_registration();
}

/// 把 logo.ico 编译进 exe 资源段（文件图标）
fn embed_app_icon() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("src/logo/logo.ico");
    res.compile().expect("failed to embed application icon");
    println!("cargo:rerun-if-changed=src/logo/logo.ico");
}

/// GUI 子系统：不弹控制台窗口；入口仍是 Rust main（mainCRTStartup）
fn set_gui_subsystem() {
    println!("cargo:rustc-link-arg-bins=/SUBSYSTEM:WINDOWS");
    println!("cargo:rustc-link-arg-bins=/ENTRY:mainCRTStartup");
}

/// 解析 prl 文件并链接静态 Qt 的完整依赖链
///
/// qtrs 自身的 build.rs 只链了 4 个 Qt 模块库，静态链接下还会缺：
/// - 系统库（d3d11 / d2d1 / dwmapi / uiautomationcore ...）
/// - Qt bundled 库（pcre2 / freetype / harfbuzz / zlib ...）
/// - 模块内嵌资源的 .obj（qrc 初始化）
/// - qwindows 平台插件
/// 以上全部登记在各 prl 文件的 QMAKE_PRL_LIBS 里，此处统一解析。
fn link_static_qt() {
    let qt = PathBuf::from(qt_prefix());
    let libs = qt.join("lib");
    let plugins = qt.join("plugins");

    // 依赖顺序：被依赖者靠后解析即可（MSVC 链接器支持乱序库），
    // qwindows 插件放最后，其依赖的 Qt 库在前面已出现
    let prl_files = [
        libs.join("Qt6Widgets.prl"),
        libs.join("Qt6UiTools.prl"),
        plugins.join("platforms").join("qwindows.prl"),
        plugins.join("platforms").join("qminimal.prl"),
        plugins.join("iconengines").join("qsvgicon.prl"),
    ];

    // lib 搜索路径
    println!("cargo:rustc-link-search=native={}", libs.display());
    println!("cargo:rustc-link-search=native={}", plugins.join("platforms").display());
    println!("cargo:rustc-link-search=native={}", plugins.join("iconengines").display());

    let mut seen = std::collections::HashSet::new();
    for prl in &prl_files {
        for entry in parse_prl_libs(prl) {
            let entry = entry.replace("$$[QT_INSTALL_LIBS]", &libs.to_string_lossy());
            let entry = entry.replace("$$[QT_INSTALL_PREFIX]", &qt_prefix());
            let entry = entry.replace("$$[QT_INSTALL_PLUGINS]", &plugins.to_string_lossy());

            if entry.starts_with("-l") {
                // 系统库（icuuc/icuin 来自 Windows SDK 的 winsdkicu，LIB 路径可解析）
                let name = entry.trim_start_matches("-l");
                if seen.insert(entry.clone()) {
                    println!("cargo:rustc-link-lib={}", name);
                }
            } else if entry.ends_with(".obj") || entry.ends_with(".lib") {
                // 完整路径的 .obj（qrc 资源初始化）或 .lib，直接放上链接行
                if seen.insert(entry.clone()) {
                    println!("cargo:rustc-link-arg-bins={}", entry);
                }
            }
        }
    }

    // qwindows 插件本体（prl 里以完整路径出现，这里统一为 static lib 链接）
    if seen.insert("qwindows".to_string()) {
        println!("cargo:rustc-link-lib=static=qwindows");
    }

    for prl in &prl_files {
        println!("cargo:rerun-if-changed={}", prl.display());
    }
}

/// 从 prl 文件提取 QMAKE_PRL_LIBS 的条目列表
fn parse_prl_libs(prl: &Path) -> Vec<String> {
    let content = std::fs::read_to_string(prl)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", prl.display(), e));
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("QMAKE_PRL_LIBS = ") {
            // Qt6 的 prl 变量引用已在上面替换；这里按空白切分
            return rest.split_whitespace().map(|s| s.to_string()).collect();
        }
    }
    panic!("QMAKE_PRL_LIBS not found in {}", prl.display());
}

/// 编译静态插件注册 TU（Q_IMPORT_PLUGIN），没有它运行时无法初始化平台插件
fn compile_plugin_registration() {
    let qt = PathBuf::from(qt_prefix());
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .file("src/cpp/plugin_registration.cpp")
        .include(qt.join("include"))
        .include(qt.join("include/QtCore"))
        .flag("/DQT_STATIC")
        .flag("/std:c++17")
        .flag("/Zc:__cplusplus")
        .flag("/permissive-")
        .flag("/utf-8")
        .compile("qt_static_plugins");
    println!("cargo:rerun-if-changed=src/cpp/plugin_registration.cpp");
}
