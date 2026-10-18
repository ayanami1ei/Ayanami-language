//! mcl.1：TokenStream 编解码（ABI v6，设计见 docs/macros.md §3.1）。
//!
//! 编码（小端）：`[kind:u8][file:u32][line:u32][col:u32][len:u32][bytes...]` 序列。
//! v1 为**扁平** token 序列（分组 token 暂以标点承载；GROUP kind 预留）。
//! 文本载体为 `String`（AyaBuf data+len，二进制安全——宏侧可直接用 `+` 拼接）。
use crate::error::{Error, Result};

pub(crate) const TOK_IDENT: u8 = 0;
pub(crate) const TOK_LITERAL: u8 = 1;
pub(crate) const TOK_PUNCT: u8 = 2;
pub(crate) const TOK_KEYWORD: u8 = 3;
/// 预留：分组 token（`(...)`/`[...]`/`{...}`，递归承载子流）
pub(crate) const TOK_GROUP: u8 = 4;

const HEADER: usize = 17; // kind(1) + file/line/col(4×3) + len(4)

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Token {
    pub kind: u8,
    pub file: u32,
    pub line: u32,
    pub col: u32,
    pub text: String,
    /// 分组 token（`(...)`/`[...]`/`{...}`）的子流（编码为内层字节；见 encode/decode）
    pub children: Option<Vec<Token>>,
}

/// 源码 → token 流（复用 lexer；EOF 剔除；span 为源码位置；成对括号折叠为分组 token）。
pub(crate) fn lex_to_tokens(src: &str) -> Vec<Token> {
    let mut lexer = crate::lexer::Lexer::new(src);
    let flat: Vec<Token> = lexer.tokenize_all().into_iter()
        .filter(|t| !matches!(t.kind, crate::lexer::TokenKind::EOF))
        .map(|t| {
            let (kind, text) = classify(&t.kind);
            let span = t.span();
            Token {
                kind,
                file: 0, // v1：Span 不含文件索引（编解码字段预留）
                line: span.start_line as u32,
                col: span.start_col as u32,
                text,
                children: None,
            }
        })
        .collect();
    group_tokens(flat)
}

/// 扁平 token 序列 → 成对括号折叠为分组（`(`/`[`/`{` 开组，对应闭符收组）。
fn group_tokens(flat: Vec<Token>) -> Vec<Token> {
    fn close_of(open: &str) -> Option<&'static str> {
        match open {
            "(" => Some(")"),
            "[" => Some("]"),
            "{" => Some("}"),
            _ => None,
        }
    }
    let mut out: Vec<Token> = Vec::new();
    let mut stack: Vec<(Token, Vec<Token>)> = Vec::new(); // (开组符, 已收集子流)
    for t in flat {
        if t.kind == TOK_PUNCT {
            if let Some(_c) = close_of(&t.text) {
                let mut g = t;
                g.kind = TOK_GROUP;
                stack.push((g, Vec::new()));
                continue;
            }
            let closes = matches!(t.text.as_str(), ")" | "]" | "}");
            if closes {
                if let Some((mut g, children)) = stack.pop() {
                    g.children = Some(group_tokens(children));
                    let target = &mut if stack.is_empty() { &mut out } else { &mut stack.last_mut().unwrap().1 };
                    target.push(g);
                    continue;
                }
            }
        }
        let target = &mut if stack.is_empty() { &mut out } else { &mut stack.last_mut().unwrap().1 };
        target.push(t);
    }
    // 未闭合的开组符降级回标点（容错）
    while let Some((g, children)) = stack.pop() {
        let mut g = g;
        g.kind = TOK_PUNCT;
        g.children = None;
        let target = &mut if stack.is_empty() { &mut out } else { &mut stack.last_mut().unwrap().1 };
        target.push(g);
        target.extend(children);
    }
    out
}

fn classify(kind: &crate::lexer::TokenKind) -> (u8, String) {
    use crate::lexer::TokenKind as K;
    match kind {
        K::Identifier(s) => (TOK_IDENT, s.clone()),
        K::Keyword(k) => (TOK_KEYWORD, format!("{}", k)),
        // 字面量：字符串按原文重新转义（lexer 存的是去转义内容）
        K::StringLiteral(s) => (TOK_LITERAL, format!("\"{}\"", crate::formatter::escape_string(s))),
        K::IntLiteral(v) => (TOK_LITERAL, v.clone()),
        K::FloatLiteral(v) => (TOK_LITERAL, v.clone()),
        K::CharLiteral(c) => (TOK_LITERAL, format!("'{}'", c)),
        K::Delimiter(d) => (TOK_PUNCT, format!("{}", d)),
        K::Operator(o) => (TOK_PUNCT, o.clone()),
        K::EOF => (TOK_PUNCT, String::new()),
    }
}

/// token 流 → 编码字节流（分组 token 的负载为递归编码的子流）。
pub(crate) fn encode(tokens: &[Token]) -> Vec<u8> {
    let mut out = Vec::new();
    for t in tokens {
        let payload: Vec<u8> = match &t.children {
            Some(children) => {
                // 分组：首字节为开符码（`(`/`[`/`{`），其后为递归编码的子流
                let code = match t.text.as_str() {
                    "(" => 0u8,
                    "[" => 1u8,
                    "{" => 2u8,
                    _ => 3u8,
                };
                let mut p = vec![code];
                p.extend(encode(children));
                p
            }
            None => t.text.as_bytes().to_vec(),
        };
        out.push(t.kind);
        out.extend_from_slice(&t.file.to_le_bytes());
        out.extend_from_slice(&t.line.to_le_bytes());
        out.extend_from_slice(&t.col.to_le_bytes());
        out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
    }
    out
}

/// 解码字节流（非法编码 → 位置化错误）。
pub(crate) fn decode(bytes: &[u8]) -> Result<Vec<Token>> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if i + HEADER > bytes.len() {
            return Err(Error::Compile(format!(
                "token stream truncated at byte {} (header needs {} bytes)",
                i, HEADER
            )));
        }
        let kind = bytes[i];
        let rd = |o: usize| u32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
        let file = rd(i + 1);
        let line = rd(i + 5);
        let col = rd(i + 9);
        let len = rd(i + 13) as usize;
        let start = i + HEADER;
        if start + len > bytes.len() {
            return Err(Error::Compile(format!(
                "token stream truncated: token at byte {} claims {} bytes, only {} left",
                i, len, bytes.len() - start
            )));
        }
        if kind > TOK_GROUP {
            return Err(Error::Compile(format!(
                "token stream: unknown token kind {} at byte {}", kind, i
            )));
        }
        let payload = &bytes[start..start + len];
        let (text, children) = if kind == TOK_GROUP {
            let delim = match payload.first().copied().unwrap_or(3) {
                0 => "(",
                1 => "[",
                2 => "{",
                _ => "",
            };
            let rest = if payload.is_empty() { &payload[..] } else { &payload[1..] };
            (delim.to_string(), Some(decode(rest)?))
        } else {
            let text = String::from_utf8(payload.to_vec())
                .map_err(|_| Error::Compile(format!("token stream: invalid UTF-8 at byte {}", start)))?;
            (text, None)
        };
        out.push(Token { kind, file, line, col, text, children });
        i = start + len;
    }
    Ok(out)
}

/// token 流 → 源码（空格分隔；分组还原为「开符 + 子流 + 闭符」；
/// 字面量/标点按原文——字符串字面量含引号整体成 token）。
pub(crate) fn to_source(tokens: &[Token]) -> String {
    let mut out: Vec<String> = Vec::new();
    for t in tokens {
        match &t.children {
            Some(children) => {
                let close = match t.text.as_str() {
                    "(" => ")",
                    "[" => "]",
                    "{" => "}",
                    _ => "",
                };
                out.push(format!("{} {} {}", t.text, to_source(children), close));
            }
            None => out.push(t.text.clone()),
        }
    }
    out.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encode_decode() {
        let src = "fn f() -> int { return 42 }";
        let toks = lex_to_tokens(src);
        let bytes = encode(&toks);
        let back = decode(&bytes).expect("decode");
        assert_eq!(toks, back);
        assert_eq!(to_source(&back), "fn f (  ) -> int { return 42 }");
    }

    #[test]
    fn decode_rejects_truncated() {
        let toks = lex_to_tokens("fn f() -> int { return 1 }");
        let bytes = encode(&toks);
        assert!(decode(&bytes[..bytes.len() - 3]).is_err());
    }

    #[test]
    fn group_roundtrip() {
        let src = "fn f(a: int) -> int { return a }";
        let toks = lex_to_tokens(src);
        // `(a: int)` 与 `{ return a }` 折叠为分组
        assert!(toks.iter().any(|t| t.kind == TOK_GROUP && t.text == "("));
        let bytes = encode(&toks);
        let back = decode(&bytes).expect("decode");
        assert_eq!(toks, back);
        let src2 = to_source(&back);
        assert!(src2.contains("( a : int )"));
        assert!(src2.contains("{ return a }"));
    }

    #[test]
    fn binary_safe_string_literal() {
        let toks = lex_to_tokens("x = \"a b\"");
        let bytes = encode(&toks);
        let back = decode(&bytes).expect("decode");
        assert!(back.iter().any(|t| t.text == "\"a b\""));
    }
}
