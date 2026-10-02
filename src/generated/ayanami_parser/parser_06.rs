use super::*;

impl Parser {
    pub fn ppattern(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Pattern");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("(")?;
            if let asuka::runtime::Value::Node(child) = self.ppattern_args()? {
                n.set("pattern_args", asuka::runtime::Value::Node(child));
            }
            self.0.expect(")")?;
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn ppattern_args(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("PatternArgs");
        if let asuka::runtime::Value::Node(child) = self.ppattern()? {
            n.set("pattern", asuka::runtime::Value::Node(child));
        }
        loop {
            let _gr_saved = self.0.pos;
            if let Ok(_) = (|| -> Result<(), String> {
                    { // group
                    let _g_saved = self.0.pos;
                        self.0.expect(",")?;
                        if let asuka::runtime::Value::Node(child) = self.ppattern()? {
                            n.set("pattern", asuka::runtime::Value::Node(child));
                        }
                    } // end group
                Ok(())
        })() {}
            else { self.0.pos = _gr_saved; break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pexpr(&mut self) -> Result<asuka::runtime::Value, String> {
        if let Ok(val) = self.pbinary_expr() { return Ok(val); }
        if let Ok(val) = self.punary_expr() { return Ok(val); }
        if let Ok(val) = self.pfn_call_expr() { return Ok(val); }
        if let Ok(val) = self.pcall_expr() { return Ok(val); }
        if let Ok(val) = self.pfield_expr() { return Ok(val); }
        if let Ok(val) = self.pindex_expr() { return Ok(val); }
        if let Ok(val) = self.pmethod_call_expr() { return Ok(val); }
        if let Ok(val) = self.pmatch_expr() { return Ok(val); }
        if let Ok(val) = self.pif_expr() { return Ok(val); }
        if let Ok(val) = self.pblock_expr() { return Ok(val); }
        if let Ok(val) = self.plambda_expr() { return Ok(val); }
        if let Ok(val) = self.pmove_expr() { return Ok(val); }
        if let Ok(val) = self.pclone_expr() { return Ok(val); }
        if let Ok(val) = self.pto_unique_expr() { return Ok(val); }
        if let Ok(val) = self.pto_shared_expr() { return Ok(val); }
        if let Ok(val) = self.pto_weak_expr() { return Ok(val); }
        if let Ok(val) = self.pref_expr() { return Ok(val); }
        if let Ok(val) = self.ptry_op() { return Ok(val); }
        if let Ok(val) = self.pstruct_literal() { return Ok(val); }
        if let Ok(val) = self.parray_literal() { return Ok(val); }
        if let Ok(val) = self.parray_sized() { return Ok(val); }
        if let Ok(val) = self.penum_construct() { return Ok(val); }
        if let Ok(val) = self.pasm_expr() { return Ok(val); }
        if let Ok(val) = self.pnull_expr() { return Ok(val); }
        if self.0.tok().kind == "Ident" {
            let mut node = asuka::runtime::Node::new("ident");
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                node.set("ident", asuka::runtime::Value::Node(child));
            }
            return Ok(asuka::runtime::Value::Node(Box::new(node)));
        }
        if let Ok(val) = self.pint_literal() { return Ok(val); }
        if let Ok(val) = self.pfloat_literal() { return Ok(val); }
        if let Ok(val) = self.pstring_literal() { return Ok(val); }
        if let Ok(val) = self.pchar_literal() { return Ok(val); }
        if let Ok(val) = self.pbool_literal() { return Ok(val); }
        return Err(format!("no alt"));
    }

    pub fn pbinary_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        self.parse_expr_prec(0)
    }

    pub(super) fn parse_expr_prec(&mut self, min_prec: u32) -> Result<asuka::runtime::Value, String> {
        let mut lhs = self.pexpr_primary()?;
        loop {
            let tok = self.0.tok().clone();
            let prec = match tok.kind.as_str() {
                "+" => Some((5u32, 6u32)),
                "-" => Some((5u32, 6u32)),
                "*" => Some((6u32, 7u32)),
                "/" => Some((6u32, 7u32)),
                "%" => Some((6u32, 7u32)),
                "==" => Some((3u32, 4u32)),
                "!=" => Some((3u32, 4u32)),
                "<" => Some((3u32, 4u32)),
                ">" => Some((3u32, 4u32)),
                "<=" => Some((3u32, 4u32)),
                ">=" => Some((3u32, 4u32)),
                "&&" => Some((4u32, 5u32)),
                "||" => Some((4u32, 5u32)),
                "=" => Some((2u32, 2u32)),
                "!" => Some((1u32, 2u32)),
                "-" => Some((7u32, 8u32)),
                "[" => Some((1u32, 2u32)),
                "]" => Some((1u32, 2u32)),
                _ => None,
            };
            match prec {
                Some((this_prec, next_prec)) if this_prec >= min_prec => {
                    self.0.adv(); // consume operator
                    let rhs = self.parse_expr_prec(next_prec)?;
                    let mut node = asuka::runtime::Node::new("BinaryExpr");
                    node.set("lhs", lhs);
                    node.set("op", asuka::runtime::Value::String(tok.kind));
                    node.set("rhs", rhs);
                    lhs = asuka::runtime::Value::Node(Box::new(node));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    pub(super) fn pexpr_primary(&mut self) -> Result<asuka::runtime::Value, String> {
        // Try all expression alternatives except BinaryExpr
        if let Ok(val) = self.punary_expr() { return Ok(val); }
        if let Ok(val) = self.pfn_call_expr() { return Ok(val); }
        if let Ok(val) = self.pcall_expr() { return Ok(val); }
        if let Ok(val) = self.pfield_expr() { return Ok(val); }
        if let Ok(val) = self.pindex_expr() { return Ok(val); }
        if let Ok(val) = self.pmethod_call_expr() { return Ok(val); }
        if let Ok(val) = self.pmatch_expr() { return Ok(val); }
        if let Ok(val) = self.pif_expr() { return Ok(val); }
        if let Ok(val) = self.pblock_expr() { return Ok(val); }
        if let Ok(val) = self.plambda_expr() { return Ok(val); }
        if let Ok(val) = self.pmove_expr() { return Ok(val); }
        if let Ok(val) = self.pclone_expr() { return Ok(val); }
        if let Ok(val) = self.pto_unique_expr() { return Ok(val); }
        if let Ok(val) = self.pto_shared_expr() { return Ok(val); }
        if let Ok(val) = self.pto_weak_expr() { return Ok(val); }
        if let Ok(val) = self.pref_expr() { return Ok(val); }
        if let Ok(val) = self.ptry_op() { return Ok(val); }
        if let Ok(val) = self.pstruct_literal() { return Ok(val); }
        if let Ok(val) = self.parray_literal() { return Ok(val); }
        if let Ok(val) = self.parray_sized() { return Ok(val); }
        if let Ok(val) = self.penum_construct() { return Ok(val); }
        if let Ok(val) = self.pasm_expr() { return Ok(val); }
        if let Ok(val) = self.pnull_expr() { return Ok(val); }
        if let Ok(val) = self.pstruct_literal() { return Ok(val); }
        if let Ok(val) = self.pi() { return Ok(val); }
        Err("expected expression".into())
    }

    pub fn punary_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("UnaryExpr");
        { // group
        let _g_saved = self.0.pos;
            if self.0.tok().kind == "-" {
                let mut node = asuka::runtime::Node::new("-");
                self.0.expect("-")?;
                return Ok(asuka::runtime::Value::Node(Box::new(node)));
            }
            if self.0.tok().kind == "!" {
                let mut node = asuka::runtime::Node::new("!");
                self.0.expect("!")?;
                return Ok(asuka::runtime::Value::Node(Box::new(node)));
            }
            return Err(format!("no alt"));
        } // end group
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }
}
