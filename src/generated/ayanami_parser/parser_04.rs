use super::*;

impl Parser {
    pub fn pshared_type(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("SharedType");
        self.0.expect("SHARED")?;
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn punique_type(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("UniqueType");
        self.0.expect("UNIQUE")?;
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pweak_type(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("WeakType");
        self.0.expect("WEAK")?;
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pref_type(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("RefType");
        self.0.expect("REF")?;
        self.0.expect("MUT")?;
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfn_type(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FnType");
        self.0.expect("FN")?;
        self.0.expect("(")?;
        if let asuka::runtime::Value::Node(child) = self.ptype_list()? {
            n.set("type_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect(")")?;
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("->")?;
            if let asuka::runtime::Value::Node(child) = self.ptyp()? {
                n.set("typ", asuka::runtime::Value::Node(child));
            }
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn parray_type(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ArrayType");
        self.0.expect("[")?;
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect(";")?;
            if let asuka::runtime::Value::Node(child) = self.pexpr()? {
                n.set("expr", asuka::runtime::Value::Node(child));
            }
        } // end group
        self.0.expect("]")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn ptype_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("TypeList");
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        loop {
            let _gr_saved = self.0.pos;
            if let Ok(_) = (|| -> Result<(), String> {
                    { // group
                    let _g_saved = self.0.pos;
                        self.0.expect(",")?;
                        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
                            n.set("typ", asuka::runtime::Value::Node(child));
                        }
                    } // end group
                Ok(())
        })() {}
            else { self.0.pos = _gr_saved; break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pblock(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Block");
        self.0.expect("{")?;
        loop {
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pstmt()? {
                n.set("stmt", asuka::runtime::Value::Node(child));
            } else { break; }
        }
        self.0.expect("}")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pstmt(&mut self) -> Result<asuka::runtime::Value, String> {
        if let Ok(val) = self.pvar_decl() { return Ok(val); }
        if let Ok(val) = self.pfield_assign() { return Ok(val); }
        if let Ok(val) = self.pindex_assign() { return Ok(val); }
        if let Ok(val) = self.preturn_stmt() { return Ok(val); }
        if let Ok(val) = self.pif_stmt() { return Ok(val); }
        if let Ok(val) = self.pwhile_stmt() { return Ok(val); }
        if let Ok(val) = self.pfor_stmt() { return Ok(val); }
        if let Ok(val) = self.pmatch_stmt() { return Ok(val); }
        if let Ok(val) = self.pbreak_stmt() { return Ok(val); }
        if let Ok(val) = self.pcontinue_stmt() { return Ok(val); }
        if let Ok(val) = self.pexpr_stmt() { return Ok(val); }
        if let Ok(val) = self.pblock() { return Ok(val); }
        return Err(format!("no alt"));
    }

    pub fn pvar_decl(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("VarDecl");
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        self.0.expect("=")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect(";")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfield_assign(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FieldAssign");
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect(".")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        self.0.expect("=")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect(";")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }
}
