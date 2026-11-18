//! mcl.3：结构化宏展开记录（`ayanami dump macros` / IDE）。

use std::cell::{Cell, RefCell};

use serde_json::{json, Value};

use crate::error::{Error, Result};

#[derive(Clone, PartialEq, Eq)]
struct TraceToken {
    kind: u8,
    line: u32,
    col: u32,
    text: String,
}

thread_local! {
    static ENABLED: Cell<bool> = const { Cell::new(false) };
    static RECORDS: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
}

pub(crate) fn begin() {
    RECORDS.with(|records| records.borrow_mut().clear());
    ENABLED.with(|enabled| enabled.set(true));
}

pub(crate) fn record(
    kind: &str,
    name: &str,
    file: &str,
    line: usize,
    col: usize,
    token_mode: bool,
    input: &str,
    output: &str,
) {
    if !ENABLED.with(Cell::get) {
        return;
    }
    let input_tokens = flat_tokens(input);
    let output_tokens = flat_tokens(output);
    let span_map = map_tokens(&input_tokens, &output_tokens);
    RECORDS.with(|records| {
        records.borrow_mut().push(json!({
            "kind": kind,
            "name": name,
            "file": file,
            "line": line,
            "col": col,
            "token_mode": token_mode,
            "input": input,
            "output": output,
            "input_tokens": input_tokens.iter().map(token_json).collect::<Vec<_>>(),
            "output_tokens": output_tokens.iter().map(token_json).collect::<Vec<_>>(),
            "span_map": span_map,
        }));
    });
}

fn flat_tokens(source: &str) -> Vec<TraceToken> {
    let mut out = Vec::new();
    flatten(&super::token_stream::lex_to_tokens(source), &mut out);
    out
}

fn flatten(tokens: &[super::token_stream::Token], out: &mut Vec<TraceToken>) {
    for token in tokens {
        out.push(TraceToken {
            kind: token.kind,
            line: token.line,
            col: token.col,
            text: token.text.clone(),
        });
        if let Some(children) = &token.children {
            flatten(children, out);
            let close = match token.text.as_str() {
                "(" => ")",
                "[" => "]",
                "{" => "}",
                _ => "",
            };
            out.push(TraceToken {
                kind: super::token_stream::TOK_PUNCT,
                line: token.line,
                col: token.col,
                text: close.to_string(),
            });
        }
    }
}

fn token_json(token: &TraceToken) -> Value {
    json!({
        "kind": token.kind,
        "line": token.line,
        "col": token.col,
        "text": token.text,
    })
}

fn map_tokens(input: &[TraceToken], output: &[TraceToken]) -> Vec<Value> {
    let mut dp = vec![vec![0usize; output.len() + 1]; input.len() + 1];
    for i in (0..input.len()).rev() {
        for j in (0..output.len()).rev() {
            dp[i][j] = if same_token(&input[i], &output[j]) {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    let (mut i, mut j, mut out) = (0usize, 0usize, Vec::new());
    while i < input.len() && j < output.len() {
        if same_token(&input[i], &output[j]) {
            out.push(json!({"output": j, "input": i}));
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    out
}

fn same_token(a: &TraceToken, b: &TraceToken) -> bool {
    a.kind == b.kind && a.text == b.text
}

pub(crate) fn finish() -> Result<String> {
    ENABLED.with(|enabled| enabled.set(false));
    let records = RECORDS.with(|records| std::mem::take(&mut *records.borrow_mut()));
    serde_json::to_string_pretty(&json!({
        "schema": 1,
        "expansions": records,
    }))
    .map_err(|e| Error::Compile(format!("macro trace JSON encode failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structured_trace_roundtrip() {
        begin();
        record("item", "identity", "test.aya", 3, 1, false, "same", "same");
        let json = finish().expect("trace json");
        let value: Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(value["schema"], 1);
        assert_eq!(value["expansions"][0]["name"], "identity");
        assert_eq!(value["expansions"][0]["line"], 3);
        assert_eq!(value["expansions"][0]["input"], "same");
        assert_eq!(value["expansions"][0]["span_map"][0]["output"], 0);
    }
}
