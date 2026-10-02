use super::*;

impl Parser {
    pub fn pgeneric_args(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("GenericArgs");
        self.0.expect("[")?;
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
        self.0.expect("]")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pgeneric_params(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("GenericParams");
        self.0.expect("[")?;
        if let asuka::runtime::Value::Node(child) = self.pgeneric_param()? {
            n.set("generic_param", asuka::runtime::Value::Node(child));
        }
        loop {
            let _gr_saved = self.0.pos;
            if let Ok(_) = (|| -> Result<(), String> {
                    { // group
                    let _g_saved = self.0.pos;
                        self.0.expect(",")?;
                        if let asuka::runtime::Value::Node(child) = self.pgeneric_param()? {
                            n.set("generic_param", asuka::runtime::Value::Node(child));
                        }
                    } // end group
                Ok(())
        })() {}
            else { self.0.pos = _gr_saved; break; }
        }
        self.0.expect("]")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pgeneric_param(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("GenericParam");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect(":")?;
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("ident", asuka::runtime::Value::Node(child));
            }
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn ptyp(&mut self) -> Result<asuka::runtime::Value, String> {
        if let Ok(val) = self.ptype_base() { return Ok(val); }
        if let Ok(val) = self.punique_type() { return Ok(val); }
        if let Ok(val) = self.pfn_type() { return Ok(val); }
        if let Ok(val) = self.parray_type() { return Ok(val); }
        if let Ok(val) = self.pref_type() { return Ok(val); }
        return Err(format!("no alt"));
    }

    pub fn ptype_base(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("TypeBase");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            if let asuka::runtime::Value::Node(child) = self.pgeneric_args()? {
                n.set("generic_args", asuka::runtime::Value::Node(child));
            }
        } // end group
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
}
