use std::collections::{HashMap, HashSet, BTreeMap};
use std::fmt::Write;

use crate::hir::ir::{FnId, HirLiteral, HirType, VarId};
use crate::intern::Symbol;
use crate::mir::ir::MirLocal;
use crate::parser::ast::{BinaryOp, UnaryOp};

/// Dynamic IR node — for user-defined extensions.
/// NOT an enum variant — attached to parent structs via `custom: Vec<IrNode>`.
#[derive(Debug, Clone)]
pub struct IrNode {
    pub kind: String,
    pub fields: BTreeMap<String, IrValue>,
}

#[derive(Debug, Clone)]
pub enum IrValue {
    Node(Box<IrNode>),
    Nodes(Vec<IrNode>),
    U64(u64),
    I64(i64),
    String(String),
    None,
}

impl IrNode {
    pub fn new(kind: &str) -> Self {
        Self { kind: kind.to_string(), fields: BTreeMap::new() }
    }
    pub fn set(&mut self, name: &str, val: IrValue) -> &mut Self {
        self.fields.insert(name.to_string(), val);
        self
    }
    pub fn get(&self, name: &str) -> Option<&IrValue> {
        self.fields.get(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvKind {
    ToUnique,
}

#[derive(Debug, Clone)]
pub enum LirValue {
    Var(VarId),
    Tmp(u64),
    Param(u64),
    Literal(HirLiteral, HirType),
}

// ═══════════════════════════════════════════════════════════════════
//  LirEmitCtx — context provided to each instruction's emit()
//  Carries program info, temp counter, and helper methods.
// ═══════════════════════════════════════════════════════════════════

pub struct LirEmitCtx<'a> {
    pub prog: &'a LirProgram,
    pub load_tmp: u64,
    pub current_fn_ret_ty: HirType,
}

impl LirEmitCtx<'_> {
    pub fn llvm_type(&self, ty: &HirType) -> String {
        match ty {
            HirType::Int => "i64".into(),
            HirType::Float => "double".into(),
            HirType::Char => "i8".into(),
            HirType::Bool => "i1".into(),
            HirType::Void => "void".into(),
            HirType::Named(s) => {
                if self.prog.struct_defs.contains_key(s) {
                    return format!("%struct.{}", sanitize_name(&s.as_str()));
                }
                // 泛型实例名（Result<int,int>）回退到基名（Result）
                let name = s.as_str();
                if let Some(pos) = name.find('<').or_else(|| name.find('[')) {
                    let base = crate::intern::Symbol::intern(&name[..pos]);
                    if self.prog.struct_defs.contains_key(&base) {
                        return format!("%struct.{}", sanitize_name(&name[..pos]));
                    }
                }
                "i8*".into()
            }
            HirType::Unique(inner) => {
                if matches!(inner.as_ref(), HirType::Named(_) | HirType::FatPtr { .. }) {
                    "ptr".into()
                } else {
                    self.llvm_type(inner)
                }
            }
            HirType::FatPtr { .. } => "{ ptr, ptr }".into(),
            HirType::Array(_) | HirType::ArraySized(_, _) => "ptr".into(),
            HirType::FnPtr(..) => "ptr".into(),
            HirType::Ref(_, _) => "ptr".into(),
        }
    }

    pub fn value_ref(&self, val: &LirValue, expected_ty: &HirType) -> String {
        match val {
            LirValue::Tmp(t) => format!("%t{}", t),
            LirValue::Param(i) => format!("%{}", i),
            LirValue::Var(v) => format!("%v{}", v.0),
            LirValue::Literal(lit, _) => lit_to_string(lit, expected_ty),
        }
    }

    pub fn tmp(&mut self) -> u64 {
        let t = self.load_tmp;
        self.load_tmp += 1;
        t
    }

    pub fn struct_llvm_name(&self, name: &Symbol) -> Option<String> {
        if self.prog.struct_defs.contains_key(name) {
            return Some(format!("%struct.{}", sanitize_name(&name.as_str())));
        }
        let s = name.as_str();
        let base = s.find('<').or_else(|| s.find('[')).map(|p| &s[..p]);
        if let Some(base) = base {
            let base_sym = Symbol::intern(base);
            if self.prog.struct_defs.contains_key(&base_sym) {
                return Some(format!("%struct.{}", sanitize_name(base)));
            }
        }
        None
    }
}

// ═══════════════════════════════════════════════════════════════════
//  LirNode trait
// ═══════════════════════════════════════════════════════════════════

pub trait LirNode: std::fmt::Debug {
    fn clone_node(&self) -> Box<dyn LirNode>;
    fn kind(&self) -> &'static str;
    fn as_any(&self) -> &dyn std::any::Any;
    fn emit(&self, ctx: &mut LirEmitCtx) -> Vec<String>;
    fn display(&self, f: &mut dyn Write) -> std::fmt::Result;
    fn serialize(&self, buf: &mut Vec<u8>);
}

// ═══════════════════════════════════════════════════════════════════
//  LirNodeBox — cloneable wrapper
// ═══════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct LirNodeBox(pub Box<dyn LirNode>);

impl Clone for LirNodeBox {
    fn clone(&self) -> Self {
        LirNodeBox(self.0.clone_node())
    }
}

impl std::ops::Deref for LirNodeBox {
    type Target = dyn LirNode;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl From<SLirCustom> for LirNodeBox {
    fn from(v: SLirCustom) -> Self { LirNodeBox(Box::new(v)) }
}

// ═══════════════════════════════════════════════════════════════════
//  s_lir! macro
// ═══════════════════════════════════════════════════════════════════

macro_rules! s_lir {
    ($name:ident { $($field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Debug, Clone)]
        pub struct $name { $(pub $field: $ty),* }
    };
}

macro_rules! impl_into_lir_node_box {
    ($($ty:ident),* $(,)?) => {
        $(impl From<$ty> for LirNodeBox {
            fn from(v: $ty) -> Self { LirNodeBox(Box::new(v) as Box<dyn LirNode>) }
        })*
    };
}

mod helpers;
mod nodes;
mod nodes_a;
mod nodes_b;
mod nodes_c;
mod nodes_d;
mod nodes_e;

pub use helpers::*;
pub use nodes::*;
pub use nodes_d::*;
