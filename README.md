# Qt6StaticRust

Qt6 静态链接的 Rust 桌面程序通用框架。基于 [qtrs](https://crates.io/crates/qtrs)（Qt6 Widgets 的 Rust 绑定），
单一 exe、零 DLL 依赖、双击即用，适合作为中小型 Windows 桌面工具的起点。

## 特性

- **Qt6 静态链接**：Qt6Core/Gui/Widgets 及 qwindows 平台插件全部编译进 exe，发布无需携带任何 Qt DLL
- **GUI 子系统**：双击运行不弹控制台；exe 内嵌图标（`src/logo/logo.ico`）
- **日志**：Debug 构建写 exe 同目录 `running.log`（中国时区时间戳，RUST_LOG 可覆盖级别）；
  Release 构建日志宏全部编译为空，零开销
- **单实例防护**：命名互斥体实现，可在配置中开关；重复启动时自动激活已有窗口
- **窗口框架**：代码直接构建主窗口（无 .ui 文件依赖），QSS 集中管理全局样式，
  默认尺寸可配置、窗口可自由拉伸、启动时屏幕居中
- **vendored qtrs**：`vendor/qtrs` 为修补版 qtrs（支持 QMAKE/QT_PREFIX 环境变量指定静态 Qt），
  通过 `[patch.crates-io]` 生效

## 环境要求

- Windows + MSVC 工具链（`cl.exe`、Windows SDK）
- Rust（edition 2021）
- 静态编译的 Qt 6（含 `bin/qmake.exe`、`lib/*.prl`、`plugins/`）
- 无需 libclang / sqlite 等其他依赖

## 构建与运行

```bat
cargo build
target\debug\Qt6StaticRust.exe
```

换机器 / 换 Qt 版本时，只需修改 `.cargo/config.toml` 中的两个路径：

```toml
[env]
QT_PREFIX = "D:\\qt-everywhere-src-6.11.0\\static"   # 静态 Qt 根目录（build.rs 链接用）
QMAKE   = "D:\\qt-everywhere-src-6.11.0\\static\\bin\\qmake.exe"  # vendored qtrs 编译用
```

## 配置说明

全局配置单一数据源在 [src/config.rs](src/config.rs)，运行参数都在这里改：

| 字段 | 说明 | 默认值 |
|---|---|---|
| `app_name` | 窗口标题等显示名 | `"Qt6StaticRust"` |
| `process_name` | 进程/二进制名，单实例锁名由它派生（`Local\{process_name}SingleInstance`），改名即隔离，不与其他应用冲突 | `"Qt6StaticRust"` |
| `version` | 版本号，自动跟随 `Cargo.toml` | — |
| `single_instance` | 单实例防护开关：`true` 时重复启动会激活已有窗口并退出 | `true` |
| `window_width` / `window_height` | 主窗口默认尺寸（窗口可自由调整） | `720` × `480` |

## 文件结构

```text
├── Cargo.toml              # 包名 qt6staticrust / 二进制名 Qt6StaticRust
├── build.rs                # 构建脚本（见下）
├── .cargo/config.toml      # QT_PREFIX / QMAKE / CXXFLAGS 构建环境（唯一需要按机器修改的地方）
├── src/
│   ├── main.rs             # 启动流程：日志 → 单实例 → 静态插件注册 → 主窗口 → 事件循环
│   ├── config.rs           # 全局配置（单一数据源，见上表）
│   ├── logging.rs          # tracing 日志初始化（Debug 写 running.log，Release 为空实现）
│   ├── ui/
│   │   ├── mod.rs          # 主窗口构建：代码直建 + 空布局作为业务控件挂载点
│   │   └── qss.rs          # 全局 QSS 样式集中管理
│   ├── cpp/
│   │   └── plugin_registration.cpp  # Q_IMPORT_PLUGIN 静态插件注册 TU
│   └── logo/               # 应用图标占位资产（logo.ico → exe 图标，logo.png → 窗口图标）
└── vendor/qtrs/            # 修补版 qtrs（支持环境变量指定静态 Qt），勿改动
```

### build.rs 做了什么

1. `winresource` 把 logo.ico 编译进 exe 资源段，并设置 `/SUBSYSTEM:WINDOWS`（不弹控制台）
2. 解析静态 Qt 各 `.prl` 文件的 `QMAKE_PRL_LIBS`，补齐 qtrs 未链接的完整依赖链
   （系统库 / Qt bundled 库 / qrc 资源对象 / qwindows 插件）——静态链接的关键步骤
3. 用 `cc` 编译 `plugin_registration.cpp`（没有它运行时无法初始化平台插件）

## 二次开发

- **加业务控件**：在 `src/ui/mod.rs` 的 `layout` 上添加，样式写进 `ui/qss.rs`
- **加全局参数**：在 `src/config.rs` 的 `Config` 结构体里加字段，全框架从此取值
- **换品牌**：替换 `src/logo/` 下图标 + 改 `config.rs` 两个名称字段即可

## 许可

仅用于内部开发参考；Qt 遵循其自身许可（LGPL/GPL/Commercial），静态链接 Qt 请注意合规。
