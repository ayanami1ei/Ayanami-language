use super::{AnnKind, AnnotationTables, Attr, Error, Result};

/// 解析并校验宏标注 → (lcl 路径, 宏名, 实参源码文本, 是否 token 宏)
pub(super) fn resolve_macro_call(
    tables: &AnnotationTables,
    attr: &Attr,
) -> Result<(String, String, Vec<String>, bool)> {
    let args: Vec<String> = attr.args.iter().map(crate::formatter::format_attr_arg).collect();
    match tables.resolve(attr)? {
        Some((pkg, macro_name, AnnKind::Macro, tokens)) => {
            let lcl_path = tables.lcl_path(&pkg).ok_or_else(|| {
                Error::Compile(format!("package `{}` has no annotation table", pkg))
            })?;
            Ok((lcl_path, macro_name, args, tokens))
        }
        Some((pkg, name, AnnKind::Transform, _)) => Err(Error::Compile(format!(
            "`#[{}]` is a HIR transform annotation (`{}::{}`), not a macro/attr (at {}:{})",
            attr.path_str(), pkg, name, attr.span.start_line, attr.span.start_col
        ))),
        Some((pkg, name, AnnKind::Pass, _)) | Some((pkg, name, AnnKind::Check, _)) => {
            Err(Error::Compile(format!(
                "`#[{}]` is a MIR annotation (`{}::{}`), not a macro/attr (at {}:{})",
                attr.path_str(), pkg, name, attr.span.start_line, attr.span.start_col
            )))
        }
        Some((pkg, attr_name, AnnKind::Attr, _)) => {
            let lcl_path = tables.lcl_path(&pkg).ok_or_else(|| {
                Error::Compile(format!("package `{}` has no annotation table", pkg))
            })?;
            Ok((lcl_path, attr_name, args, false))
        }
        None => Err(Error::Compile(format!(
            "macro #[{}] cannot be resolved (at {}:{})",
            attr.path_str(), attr.span.start_line, attr.span.start_col
        ))),
    }
}
