unsafe extern "C++" {
    include!("src/cpp/qt_widget.h");
    // --- QLabel ---
    unsafe fn QLabel_new(text: &CxxString, parent: *mut QWidget) -> *mut QLabel;
    unsafe fn QLabel_setText(label: *mut QLabel, text: &CxxString);
    unsafe fn QLabel_delete(label: *mut QLabel);
    unsafe fn QLabel_setPixmapFromIcon(label: *mut QLabel, icon_path: &CxxString, w: i32, h: i32);
}
