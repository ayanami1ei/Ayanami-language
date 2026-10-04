//! M1.5：字面量文本解析（进制 / 下划线 / 后缀拆分）

/// M1.5：拆分字面量文本为（数值部分, 可选后缀），支持 0x/0b/0o 与下划线
pub(super) fn split_literal_suffix(s: &str) -> (&str, Option<&str>) {
    let b = s.as_bytes();
    let mut i = 0;
    if b.len() >= 2 && b[0] == b'0' && matches!(b[1], b'x' | b'X' | b'b' | b'B' | b'o' | b'O') {
        i = 2;
        while i < b.len() && (b[i].is_ascii_hexdigit() || b[i] == b'_') { i += 1; }
    } else {
        while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_' || b[i] == b'.') { i += 1; }
        if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
            let mut j = i + 1;
            if j < b.len() && (b[j] == b'+' || b[j] == b'-') { j += 1; }
            if j < b.len() && b[j].is_ascii_digit() {
                while j < b.len() && (b[j].is_ascii_digit() || b[j] == b'_') { j += 1; }
                i = j;
            }
        }
    }
    if i < b.len() { (&s[..i], Some(&s[i..])) } else { (s, None) }
}

/// 解析整数字面量文本（已去下划线），支持进制前缀
pub(super) fn parse_int_text(s: &str) -> Option<i64> {
    if let Some(rest) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        i64::from_str_radix(rest, 16).ok()
    } else if let Some(rest) = s.strip_prefix("0b").or_else(|| s.strip_prefix("0B")) {
        i64::from_str_radix(rest, 2).ok()
    } else if let Some(rest) = s.strip_prefix("0o").or_else(|| s.strip_prefix("0O")) {
        i64::from_str_radix(rest, 8).ok()
    } else {
        s.parse::<i64>().ok()
    }
}
