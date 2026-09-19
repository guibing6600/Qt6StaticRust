unsafe extern "C++" {
    include!("src/cpp/eventloop.h");
    // --- QEventLoop ---
    unsafe fn QEventLoop_new() -> *mut QEventLoop;
    unsafe fn QEventLoop_exec(loop_: *mut QEventLoop) -> i32;
    unsafe fn QEventLoop_quit(loop_: *mut QEventLoop);
    unsafe fn QEventLoop_delete(loop_: *mut QEventLoop);
}
