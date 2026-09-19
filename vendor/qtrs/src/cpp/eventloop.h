// src/cpp/eventloop.h — QEventLoop（嵌套事件循环，QDialog::exec 同机制）
#pragma once

#include <QEventLoop>

inline QEventLoop *QEventLoop_new() { return new QEventLoop(); }

inline int QEventLoop_exec(QEventLoop *loop_) { return loop_->exec(); }

inline void QEventLoop_quit(QEventLoop *loop_) { loop_->quit(); }

inline void QEventLoop_delete(QEventLoop *loop_) { delete loop_; }
