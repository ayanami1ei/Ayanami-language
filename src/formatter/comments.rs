//! #179：格式化保留注释（基于行号的 trivia 回插）。
//!
//! 简化策略：整行注释在**下一个语句/字段之前**按当前缩进输出；
//! 行尾注释追加到语句末行（分号/右花括号后）；文件尾剩余注释最后输出。
//! 表达式内部的注释会被移到下一个语句前（位置近似，不丢失）。

use std::cell::RefCell;
use std::fmt::Write;

use crate::lexer::Comment;

thread_local! {
    static STATE: RefCell<CommentState> = RefCell::new(CommentState::default());
}

#[derive(Default)]
struct CommentState {
    comments: Vec<Comment>,
    next: usize,
    active: bool,
}

/// 安装注释列表（`format_program_with_comments` 调用）。
pub(crate) fn install(comments: Vec<Comment>) {
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        st.comments = comments;
        st.next = 0;
        st.active = true;
    });
}

/// 清理状态（格式化结束）。
pub(crate) fn uninstall() {
    STATE.with(|s| {
        *s.borrow_mut() = CommentState::default();
    });
}

/// 写入某行之前：输出所有 `start_line < line` 的注释（按 `level` 缩进）。
pub(crate) fn flush_before(out: &mut String, line: usize, level: usize) {
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if !st.active {
            return;
        }
        let indent = "    ".repeat(level);
        while st.next < st.comments.len() && st.comments[st.next].span.start_line < line {
            let c = &st.comments[st.next];
            let _ = writeln!(out, "{}{}", indent, c.text);
            st.next += 1;
        }
    });
}

/// 行尾注释：`start_line == line` 且 `start_col > col` → 追加到当前行末（末尾换行前）。
pub(crate) fn flush_trailing(out: &mut String, line: usize, col: usize) {
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if !st.active {
            return;
        }
        while st.next < st.comments.len()
            && st.comments[st.next].span.start_line == line
            && st.comments[st.next].span.start_col > col
        {
            let text = st.comments[st.next].text.clone();
            if out.ends_with('\n') {
                out.pop();
            }
            out.push_str("  ");
            out.push_str(&text);
            out.push('\n');
            st.next += 1;
        }
    });
}

/// 文件尾剩余注释（不缩进）。
pub(crate) fn flush_rest(out: &mut String) {
    STATE.with(|s| {
        let mut st = s.borrow_mut();
        if !st.active {
            return;
        }
        while st.next < st.comments.len() {
            let c = &st.comments[st.next];
            let _ = writeln!(out, "{}", c.text);
            st.next += 1;
        }
    });
}
