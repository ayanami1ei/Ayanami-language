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
}

/// 源码 → token 流（复用 lexer；EOF 剔除；span 为源码位置）。
pub(crate) fn lex_to_tokens(src: &str) -> Vec<Token> {
    let mut lexer = crate::lexer::Lexer::new(src);
    let tokens = lexer.tokenize_all();
    tokens.into_iter()
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
            }
        })
        .collect()
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

/// token 流 → 编码字节流。
pub(crate) fn encode(tokens: &[Token]) -> Vec<u8> {
    let mut out = Vec::new();
    for t in tokens {
        let bytes = t.text.as_bytes();
        out.push(t.kind);
        out.extend_from_slice(&t.file.to_le_bytes());
        out.extend_from_slice(&t.line.to_le_bytes());
        out.extend_from_slice(&t.col.to_le_bytes());
        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(bytes);
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
        let text = String::from_utf8(bytes[start..start + len].to_vec())
            .map_err(|_| Error::Compile(format!("token stream: invalid UTF-8 at byte {}", start)))?;
        out.push(Token { kind, file, line, col, text });
        i = start + len;
    }
    Ok(out)
}

/// token 流 → 源码（空格分隔；字面量/标点按原文——字符串字面量含引号整体成 token）。
pub(crate) fn to_source(tokens: &[Token]) -> String {
    tokens.iter().map(|t| t.text.as_str()).collect::<Vec<_>>().join(" ")
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
        assert_eq!(to_source(&back), "fn f ( ) -> int { return 42 }");
    }

    #[test]
    fn decode_rejects_truncated() {
        let toks = lex_to_tokens("fn f() -> int { return 1 }");
        let bytes = encode(&toks);
        assert!(decode(&bytes[..bytes.len() - 3]).is_err());
    }

    #[test]
    fn binary_safe_string_literal() {
        let toks = lex_to_tokens("x = \"a b\"");
        let bytes = encode(&toks);
        let back = decode(&bytes).expect("decode");
        assert!(back.iter().any(|t| t.text == "\"a b\""));
    }
}
