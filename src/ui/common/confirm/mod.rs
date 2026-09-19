//! 全局确认弹窗（HTML 式：全窗口蒙版 + 白色卡片，同步返回结果）
//!
//! 结构（confirm.ui）：半透明蒙版根节点内居中一张白色圆角卡片，
//! 卡片上中下三段 —— 标题 / 图标+提示内容（图标可空）/ 按钮。
//!
//! 用法（类似 HTML `confirm()`，调用处同步拿到结果）：
//!
//! ```ignore
//! let ok = confirm.show(&central, "卸载设备", "确定要删除设备「camera-dy」吗？",
//!                       Icon::Warn, Buttons::OkCancel);
//! if ok { crate::vcam::del_device(); }
//! ```
//!
//! 实现要点：`show` 内部跑一个嵌套事件循环（`QEventLoop::exec`，与
//! `QDialog::exec` 同机制），按钮回调里记录结果并 `quit`，`show` 返回。
//! 蒙版是不透明事件的 QWidget 子控件，挡住主窗口全部鼠标点击。

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use qtrs::eventloop::EventLoop;
use qtrs::prelude::*;

/// 提示图标类型（None 不显示图标）
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // 公共组件：变体按调用方需要取用
pub enum Icon {
    None,
    /// 蓝色 i：普通信息提示
    Info,
    /// 琥珀色 !：需要注意的确认
    Warn,
    /// 红色 ✕：危险/破坏性操作
    Error,
}

/// 按钮组合
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // 公共组件：变体按调用方需要取用
pub enum Buttons {
    /// 只显示「确认」
    Ok,
    /// 显示「取消」「确认」
    OkCancel,
}

/// 常驻的全局确认弹窗（构建一次，多次 show）
pub struct Confirm {
    /// 蒙版根节点（覆盖宿主，show 时 set_geometry 跟随宿主尺寸）
    overlay: Widget,
    /// 蒙版铺满的宿主容器（build 时的 parent，一般为主窗口 central）
    host: Widget,
    title: Rc<RefCell<Label>>,
    msg: Rc<RefCell<Label>>,
    icon_info: Widget,
    icon_warn: Widget,
    icon_error: Widget,
    btn_ok: Rc<RefCell<PushButton>>,
    btn_cancel: Rc<RefCell<PushButton>>,
    /// 按钮点击结果：0=未定 1=确认 2=取消
    result: Rc<Cell<u8>>,
    loop_: Rc<RefCell<EventLoop>>,
}

impl Confirm {
    /// 加载 confirm.ui 并接线（parent = 主窗口 central，常驻隐藏）
    pub fn build(parent: &Widget) -> Confirm {
        let path = format!(
            "{}/src/ui/common/confirm/confirm.ui",
            env!("CARGO_MANIFEST_DIR")
        );
        let loader = UiLoader::new();
        let overlay = loader
            .load(&path, Some(parent))
            .unwrap_or_else(|| panic!("failed to load confirm.ui: {}", path));

        let get_label = |name: &str| match overlay.find(WidgetKind::Label, name) {
            Some(FoundWidget::Label(l)) => Rc::new(RefCell::new(l)),
            _ => panic!("{} not found in confirm.ui", name),
        };
        let get_widget = |name: &str| match overlay.find(WidgetKind::Any, name) {
            Some(FoundWidget::Widget(w)) => w,
            _ => panic!("{} not found in confirm.ui", name),
        };
        let get_btn = |name: &str| match overlay.find(WidgetKind::PushButton, name) {
            Some(FoundWidget::PushButton(b)) => Rc::new(RefCell::new(b)),
            _ => panic!("{} not found in confirm.ui", name),
        };

        let title = get_label("confirm-title");
        let msg = get_label("confirm-msg");
        let icon_info = get_widget("confirm-icon-info");
        let icon_warn = get_widget("confirm-icon-warn");
        let icon_error = get_widget("confirm-icon-error");
        let btn_ok = get_btn("btn-confirm-ok");
        let btn_cancel = get_btn("btn-confirm-cancel");

        let result: Rc<Cell<u8>> = Rc::new(Cell::new(0));
        let loop_: Rc<RefCell<EventLoop>> = Rc::new(RefCell::new(EventLoop::new()));

        // 按钮：记录结果 → 退出嵌套循环（show 返回）
        {
            let r = Rc::clone(&result);
            let lp = Rc::clone(&loop_);
            btn_ok.borrow_mut().connect_clicked(move || {
                r.set(1);
                lp.borrow().quit();
            });
        }
        {
            let r = Rc::clone(&result);
            let lp = Rc::clone(&loop_);
            btn_cancel.borrow_mut().connect_clicked(move || {
                r.set(2);
                lp.borrow().quit();
            });
        }

        // QUiLoader 带 parent 加载后默认可见：先隐藏，show() 时才铺满显示
        overlay.hide();

        Confirm {
            overlay,
            host: Widget::from_raw(parent.widget_ptr(), true),
            title,
            msg,
            icon_info,
            icon_warn,
            icon_error,
            btn_ok,
            btn_cancel,
            result,
            loop_,
        }
    }

    /// 显示确认弹窗并阻塞直到用户选择，`true`=确认
    ///
    /// - `icon`: 图标类型，`Icon::None` 不显示图标
    /// - `buttons`: 要显示的按钮组合
    pub fn show(&self, title: &str, msg: &str, icon: Icon, buttons: Buttons) -> bool {
        // 文案
        self.title.borrow_mut().set_text(title.to_string());
        self.msg.borrow_mut().set_text(msg.to_string());

        // 图标：单选显示 / 全部隐藏
        self.icon_info.hide();
        self.icon_warn.hide();
        self.icon_error.hide();
        match icon {
            Icon::None => {}
            Icon::Info => self.icon_info.show(),
            Icon::Warn => self.icon_warn.show(),
            Icon::Error => self.icon_error.show(),
        }

        // 按钮组合
        match buttons {
            Buttons::Ok => self.btn_cancel.borrow().hide(),
            Buttons::OkCancel => self.btn_cancel.borrow().show(),
        }

        // 蒙版铺满宿主并置顶显示
        self.overlay
            .set_geometry(0, 0, self.host.width(), self.host.height());
        self.overlay.show();
        self.overlay.raise_widget();
        self.btn_ok.borrow().set_focus();

        // 嵌套事件循环：按钮回调 quit 后返回
        self.result.set(0);
        self.loop_.borrow().exec();

        self.overlay.hide();
        self.result.get() == 1
    }
}
