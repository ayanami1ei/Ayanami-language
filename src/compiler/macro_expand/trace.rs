//! mcl.3：结构化宏展开记录（`ayanami dump macros` / IDE）。

use std::cell::{Cell, RefCell};

use serde_json::{json, Value};

use crate::error::{Error, Result};

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
        }));
    });
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
        record("item", "identity", "test.aya", 3, 1, false, "input", "output");
        let json = finish().expect("trace json");
        let value: Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(value["schema"], 1);
        assert_eq!(value["expansions"][0]["name"], "identity");
        assert_eq!(value["expansions"][0]["line"], 3);
        assert_eq!(value["expansions"][0]["input"], "input");
    }
}
