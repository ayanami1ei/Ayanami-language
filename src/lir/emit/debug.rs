//! M7：LIR → LLVM DWARF 元数据（编译单元、子程序、行位置）。

use std::path::Path;

use crate::hir::ir::HirType;
use crate::lir::ir::LirFn;
use crate::span::Span;

use super::Emitter;

pub(super) struct DebugSource {
    source_name: String,
    directory: String,
    filename: String,
}

impl DebugSource {
    fn new(path: Option<&Path>) -> Self {
        let Some(path) = path else {
            return Self {
                source_name: "<memory>".into(),
                directory: String::new(),
                filename: "<memory>".into(),
            };
        };
        let source_name = path.to_string_lossy().into_owned();
        let directory = path.parent()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let filename = path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| source_name.clone());
        Self { source_name, directory, filename }
    }
}

pub(super) fn enabled() -> bool {
    !std::env::var("AYANAMI_DEBUG_INFO")
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false)
}

fn escaped(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\22")
}

impl Emitter<'_> {
    pub(super) fn init_debug_source(&mut self, path: Option<&Path>) {
        self.debug_source = enabled().then(|| DebugSource::new(path));
    }

    pub(super) fn emit_debug_source_filename(&mut self) {
        if let Some(src) = &self.debug_source {
            let name = escaped(&src.source_name);
            self.wln(&format!("source_filename = \"{}\"", name));
        }
    }

    pub(super) fn start_debug_function(&mut self, name: &str, span: Span) -> Option<u32> {
        self.debug_source.as_ref()?;
        let line = span.start_line.max(1);
        let types_id = self.fresh_debug_id();
        self.debug_nodes.push(format!("!{} = !{{null}}", types_id));
        let type_id = self.fresh_debug_id();
        self.debug_nodes.push(format!(
            "!{} = !DISubroutineType(types: !{})",
            type_id, types_id
        ));
        let id = self.fresh_debug_id();
        self.debug_nodes.push(format!(
            "!{} = distinct !DISubprogram(name: \"{}\", scope: !1, file: !1, line: {}, type: !{}, scopeLine: {}, flags: DIFlagPrototyped, spFlags: DISPFlagDefinition, unit: !0)",
            id,
            escaped(name),
            line,
            type_id,
            line
        ));
        self.current_debug_fn = Some(id);
        self.debug_loc_cache.clear();
        Some(id)
    }

    pub(super) fn finish_debug_function(&mut self) {
        self.current_debug_fn = None;
        self.debug_loc_cache.clear();
    }

    pub(super) fn debug_location(&mut self, span: Span) -> Option<u32> {
        let scope = self.current_debug_fn?;
        let line = span.start_line.max(1);
        let col = span.start_col.max(1);
        if let Some(id) = self.debug_loc_cache.get(&(line, col)) {
            return Some(*id);
        }
        let id = self.fresh_debug_id();
        self.debug_nodes.push(format!(
            "!{} = !DILocation(line: {}, column: {}, scope: !{})",
            id, line, col, scope
        ));
        self.debug_loc_cache.insert((line, col), id);
        Some(id)
    }

    pub(super) fn debug_local_declares(&mut self, f: &LirFn) -> Vec<String> {
        let Some(scope) = self.current_debug_fn else {
            return Vec::new();
        };
        let loc = self.debug_location(f.span).unwrap_or(scope);
        let mut lines = Vec::with_capacity(f.locals.len());
        for (idx, local) in f.locals.iter().enumerate() {
            // #207：泛型实例化出的 void 局部没有栈槽，不能发 dbg.declare。
            if matches!(local.ty, HirType::Void | HirType::Never) {
                continue;
            }
            let type_id = self.debug_type(&local.ty);
            let id = self.fresh_debug_id();
            let arg = if idx < f.params.len() { format!(", arg: {}", idx + 1) } else { String::new() };
            self.debug_nodes.push(format!(
                "!{} = !DILocalVariable(name: \"{}\", scope: !{}, file: !1, line: {}, type: !{}{})",
                id,
                escaped(&local.name.as_str()),
                scope,
                f.span.start_line.max(1),
                type_id,
                arg
            ));
            lines.push(format!(
                "#dbg_declare(ptr %v{}, !{}, !DIExpression(), !{})",
                idx, id, loc
            ));
        }
        lines
    }

    fn debug_type(&mut self, ty: &HirType) -> u32 {
        let key = format!("{:?}", ty);
        if let Some(id) = self.debug_type_cache.get(&key) {
            return *id;
        }
        let id = self.fresh_debug_id();
        self.debug_type_cache.insert(key, id);
        let meta = match ty {
            HirType::Int => format!("!{} = !DIBasicType(name: \"int\", size: 64, encoding: DW_ATE_signed)", id),
            HirType::Float => format!("!{} = !DIBasicType(name: \"float\", size: 64, encoding: DW_ATE_float)", id),
            HirType::F32 => format!("!{} = !DIBasicType(name: \"f32\", size: 32, encoding: DW_ATE_float)", id),
            HirType::Char => format!("!{} = !DIBasicType(name: \"char\", size: 8, encoding: DW_ATE_unsigned_char)", id),
            HirType::Bool => format!("!{} = !DIBasicType(name: \"bool\", size: 8, encoding: DW_ATE_boolean)", id),
            HirType::Void | HirType::Never => format!("!{} = !DIBasicType(name: \"void\", encoding: DW_ATE_unsigned)", id),
            HirType::IntN { bits, signed } => format!(
                "!{} = !DIBasicType(name: \"int{}\", size: {}, encoding: {})",
                id, bits, bits, if *signed { "DW_ATE_signed" } else { "DW_ATE_unsigned" }
            ),
            HirType::Named(name) => format!(
                "!{} = distinct !DICompositeType(tag: DW_TAG_structure_type, name: \"{}\", scope: !1, file: !1, flags: DIFlagTypePassByValue)",
                id, escaped(&name.as_str())
            ),
            HirType::FatPtr { .. } | HirType::Closure(..) => format!(
                "!{} = distinct !DICompositeType(tag: DW_TAG_structure_type, name: \"{}\", size: 128, flags: DIFlagTypePassByValue)",
                id, escaped(&format!("{:?}", ty))
            ),
            _ => format!(
                "!{} = !DIDerivedType(tag: DW_TAG_pointer_type, baseType: null, size: 64, name: \"{}\")",
                id, escaped(&format!("{:?}", ty))
            ),
        };
        self.debug_nodes.push(meta);
        id
    }

    fn fresh_debug_id(&mut self) -> u32 {
        let id = self.next_debug_id;
        self.next_debug_id += 1;
        id
    }

    pub(super) fn wln_debug(&mut self, line: &str, loc: Option<u32>) {
        let Some(loc) = loc else {
            self.wln(line);
            return;
        };
        let trimmed = line.trim_start();
        let is_label = trimmed.ends_with(':');
        let is_empty = trimmed.is_empty() || trimmed == "{" || trimmed == "}";
        if is_empty || is_label || trimmed.starts_with(';') || line.contains(" !dbg !") {
            self.wln(line);
        } else {
            self.wln(&format!("{}, !dbg !{}", line, loc));
        }
    }

    pub(super) fn emit_debug_footer(&mut self) {
        let Some((filename, directory)) = self.debug_source.as_ref()
            .map(|src| (src.filename.clone(), src.directory.clone())) else {
            return;
        };
        self.wln("!llvm.dbg.cu = !{!0}");
        self.wln("!llvm.module.flags = !{!2, !3}");
        self.wln("!llvm.ident = !{!4}");
        self.wln("");
        let optimized = crate::hir::contracts::is_release();
        self.wln_fmt(format_args!(
            "!0 = distinct !DICompileUnit(language: DW_LANG_C99, file: !1, producer: \"Ayanami {}\", isOptimized: {}, runtimeVersion: 0, emissionKind: FullDebug)",
            env!("CARGO_PKG_VERSION"), optimized
        ));
        self.wln(&format!(
            "!1 = !DIFile(filename: \"{}\", directory: \"{}\")",
            escaped(&filename),
            escaped(&directory)
        ));
        self.wln("!2 = !{i32 7, !\"Dwarf Version\", i32 5}");
        self.wln("!3 = !{i32 2, !\"Debug Info Version\", i32 3}");
        self.wln(&format!(
            "!4 = !{{!\"Ayanami Compiler {}\"}}",
            env!("CARGO_PKG_VERSION")
        ));
        for node in std::mem::take(&mut self.debug_nodes) {
            self.wln(&node);
        }
    }
}
