use super::*;

impl Parser {
    pub fn pto_unique_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ToUniqueExpr");
        self.0.expect("UNIQUE")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pto_shared_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ToSharedExpr");
        self.0.expect("SHARED")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pto_weak_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ToWeakExpr");
        self.0.expect("WEAK")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pref_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("RefExpr");
        self.0.expect("REF")?;
        self.0.expect("MUT")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn ptry_op(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("TryOp");
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect("?")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pnull_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("NullExpr");
        self.0.expect("NULL")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn parray_sized(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ArraySized");
        self.0.expect("[")?;
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        self.0.expect(";")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect("]")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pstruct_literal(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("StructLiteral");
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        self.0.expect("{")?;
        if let asuka::runtime::Value::Node(child) = self.pfield_init_list()? {
            n.set("field_init_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect("}")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfield_init(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FieldInit");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        self.0.expect("=")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfield_init_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FieldInitList");
        if let asuka::runtime::Value::Node(child) = self.pfield_init()? {
            n.set("field_init", asuka::runtime::Value::Node(child));
        }
        loop {
            let _gr_saved = self.0.pos;
            if let Ok(_) = (|| -> Result<(), String> {
                    { // group
                    let _g_saved = self.0.pos;
                        self.0.expect(",")?;
                        if let asuka::runtime::Value::Node(child) = self.pfield_init()? {
                            n.set("field_init", asuka::runtime::Value::Node(child));
                        }
                    } // end group
                Ok(())
        })() {}
            else { self.0.pos = _gr_saved; break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn parray_literal(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ArrayLiteral");
        self.0.expect("[")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr_list()? {
            n.set("expr_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect("]")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pexpr_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ExprList");
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        loop {
            let _gr_saved = self.0.pos;
            if let Ok(_) = (|| -> Result<(), String> {
                    { // group
                    let _g_saved = self.0.pos;
                        self.0.expect(",")?;
                        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
                            n.set("expr", asuka::runtime::Value::Node(child));
                        }
                    } // end group
                Ok(())
        })() {}
            else { self.0.pos = _gr_saved; break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn penum_construct(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("EnumConstruct");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        self.0.expect("::")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("(")?;
            if let asuka::runtime::Value::Node(child) = self.pexpr_list()? {
                n.set("expr_list", asuka::runtime::Value::Node(child));
            }
            self.0.expect(")")?;
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }
}
