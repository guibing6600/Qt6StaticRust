//! QEventLoop 包装：嵌套事件循环
//!
//! 在已有事件循环内再跑一层（`QDialog::exec` 同机制），
//! 用于"同步 API 内部等待用户交互"的场景（如确认弹窗）。

use crate::ffi;

pub struct EventLoop {
    ptr: *mut ffi::QEventLoop,
}

impl EventLoop {
    pub fn new() -> Self {
        Self { ptr: unsafe { ffi::QEventLoop_new() } }
    }

    /// 进入嵌套事件循环，直到 `quit` 被调用
    pub fn exec(&self) -> i32 {
        unsafe { ffi::QEventLoop_exec(self.ptr) }
    }

    /// 退出嵌套事件循环
    pub fn quit(&self) {
        unsafe { ffi::QEventLoop_quit(self.ptr) }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for EventLoop {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { ffi::QEventLoop_delete(self.ptr) }
            self.ptr = std::ptr::null_mut();
        }
    }
}
