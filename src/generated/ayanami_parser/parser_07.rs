use super::*;

impl Parser {
    pub fn pfn_call_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FnCallExpr");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        self.0.expect("(")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr_list()? {
            n.set("expr_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect(")")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pcall_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("CallExpr");
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect("(")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr_list()? {
            n.set("expr_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect(")")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfield_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FieldExpr");
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect(".")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pindex_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("IndexExpr");
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect("[")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect("]")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pmethod_call_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("MethodCallExpr");
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect(".")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        self.0.expect("(")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr_list()? {
            n.set("expr_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect(")")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pmatch_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("MatchExpr");
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

    pub fn pif_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("IfExpr");
        self.0.expect("IF")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pblock()? {
            n.set("block", asuka::runtime::Value::Node(child));
        }
        loop {
            let _gr_saved = self.0.pos;
            if let Ok(_) = (|| -> Result<(), String> {
                    { // group
                    let _g_saved = self.0.pos;
                        self.0.expect("ELIF")?;
                        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
                            n.set("expr", asuka::runtime::Value::Node(child));
                        }
                        if let asuka::runtime::Value::Node(child) = self.pblock()? {
                            n.set("block", asuka::runtime::Value::Node(child));
                        }
                    } // end group
                Ok(())
        })() {}
            else { self.0.pos = _gr_saved; break; }
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("ELSE")?;
            { // group
            let _g_saved = self.0.pos;
                if let Ok(val) = self.pif_expr() { return Ok(val); }
                if let Ok(val) = self.pblock() { return Ok(val); }
                return Err(format!("no alt"));
            } // end group
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pblock_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("BlockExpr");
        if let asuka::runtime::Value::Node(child) = self.pblock()? {
            n.set("block", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn plambda_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("LambdaExpr");
        self.0.expect("(")?;
        if let asuka::runtime::Value::Node(child) = self.pparam_list()? {
            n.set("param_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect(")")?;
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("->")?;
            if let asuka::runtime::Value::Node(child) = self.ptyp()? {
                n.set("typ", asuka::runtime::Value::Node(child));
            }
        } // end group
        if let asuka::runtime::Value::Node(child) = self.pblock()? {
            n.set("block", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pmove_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("MoveExpr");
        self.0.expect("MOVE")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pclone_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("CloneExpr");
        self.0.expect("CLONE")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }
}
