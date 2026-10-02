use super::*;

impl Parser {
    pub fn pself_param(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("SelfParam");
        { // group
        let _g_saved = self.0.pos;
            if self.0.tok().kind == "SHARED" {
                let mut node = asuka::runtime::Node::new("shared");
                self.0.expect("SHARED")?;
                return Ok(asuka::runtime::Value::Node(Box::new(node)));
            }
            if self.0.tok().kind == "UNIQUE" {
                let mut node = asuka::runtime::Node::new("unique");
                self.0.expect("UNIQUE")?;
                return Ok(asuka::runtime::Value::Node(Box::new(node)));
            }
            return Err(format!("no alt"));
        } // end group
        self.0.expect("SELF")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pimpl_block(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ImplBlock");
        if let asuka::runtime::Value::Node(child) = self.pvis()? {
            n.set("vis", asuka::runtime::Value::Node(child));
        }
        self.0.expect("IMPL")?;
        if let asuka::runtime::Value::Node(child) = self.pgeneric_params()? {
            n.set("generic_params", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pgeneric_args()? {
            n.set("generic_args", asuka::runtime::Value::Node(child));
        }
        self.0.expect("{")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("method_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect("}")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pmethod(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Method");
        if let asuka::runtime::Value::Node(child) = self.pvis()? {
            n.set("vis", asuka::runtime::Value::Node(child));
        }
        self.0.expect("FN")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pgeneric_params()? {
            n.set("generic_params", asuka::runtime::Value::Node(child));
        }
        self.0.expect("(")?;
        if let asuka::runtime::Value::Node(child) = self.pself_param()? {
            n.set("self_param", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect(",")?;
            if let asuka::runtime::Value::Node(child) = self.pparam_list()? {
                n.set("param_list", asuka::runtime::Value::Node(child));
            }
        } // end group
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
        if let Ok(val) = self.pshared_type() { return Ok(val); }
        if let Ok(val) = self.punique_type() { return Ok(val); }
        if let Ok(val) = self.pweak_type() { return Ok(val); }
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
}
