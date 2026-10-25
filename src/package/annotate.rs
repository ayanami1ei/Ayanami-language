//! A5d：包导出注解名唯一性（同一包内宏 / pass / check 跨类别唯一）。
use super::*;
use crate::error::{Error, Result};

impl Package {
    /// 同一包内注解名跨类别唯一（宏 / pass / check）。
    /// 跨包同名允许（用 `pkg::name` 消歧）；同包同名会让解析依赖顺序，必须拒绝。
    pub fn validate_annotation_names(&self) -> Result<()> {
        let mut seen: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();
        for s in &self.symbols {
            let (name, kind) = match s {
                PackageSymbol::Macro { name, .. } => (name.as_str(), "macro"),
                PackageSymbol::Pass { name } => (name.as_str(), "pass"),
                PackageSymbol::Check { name } => (name.as_str(), "check"),
                _ => continue,
            };
            if let Some(prev) = seen.insert(name, kind) {
                return Err(Error::Package(format!(
                    "annotation `{}` is declared twice in this package ({} / {}); \
                     annotation names must be unique within a package",
                    name, prev, kind
                )));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_annotation_names_across_kinds_rejected() {
        let src = "#[macro]\nfn foo() { }\n#[pass]\nfn foo(int x) { }\n";
        let program = crate::parser::parse_source(src).expect("parse");
        let mut pkg = Package::new("t".into(), "0".into());
        pkg.collect_all_symbols(&program.stmts);
        assert!(
            pkg.validate_annotation_names().is_err(),
            "same name exported as macro and pass must be rejected"
        );
    }

    #[test]
    fn distinct_annotation_names_accepted() {
        let src = "#[macro]\nfn m() { }\n#[pass]\nfn p(int x) { }\n";
        let program = crate::parser::parse_source(src).expect("parse");
        let mut pkg = Package::new("t".into(), "0".into());
        pkg.collect_all_symbols(&program.stmts);
        assert!(pkg.validate_annotation_names().is_ok());
    }
}
