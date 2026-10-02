use super::*;

impl Parser {
    pub fn pmatch_stmt(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("MatchStmt");
        self.0.expect("MATCH")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect("{")?;
        loop {
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pmatch_arm()? {
                n.set("match_arm", asuka::runtime::Value::Node(child));
            } else { break; }
        }
        self.0.expect("}")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pmatch_arm(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("MatchArm");
        if let asuka::runtime::Value::Node(child) = self.ppattern()? {
            n.set("pattern", asuka::runtime::Value::Node(child));
        }
        self.0.expect("=>")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect(",")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pbreak_stmt(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("BreakStmt");
        self.0.expect("BREAK")?;
        self.0.expect(";")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pcontinue_stmt(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ContinueStmt");
        self.0.expect("CONTINUE")?;
        self.0.expect(";")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pexpr_stmt(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ExprStmt");
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect(";")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

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
}
