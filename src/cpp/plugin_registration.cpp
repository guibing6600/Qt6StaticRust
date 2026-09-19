// 静态 Qt 平台插件注册：静态链接时插件不会自动加载，
// 必须导入并链接对应插件，否则运行时报 "no Qt platform plugin could be initialized"
#include <QtPlugin>

// 窗口平台插件
Q_IMPORT_PLUGIN(QWindowsIntegrationPlugin)
Q_IMPORT_PLUGIN(QMinimalIntegrationPlugin)

// SVG 图标引擎（底部导航的 svg 图标需要）
Q_IMPORT_PLUGIN(QSvgIconPlugin)

// 供 Rust 显式调用：链接器只拉取"定义了被引用符号"的静态库成员，
// 没有这个函数，本编译单元（含上面的插件注册器）会被整个丢弃
extern "C" void qt_register_static_plugins() {
}
