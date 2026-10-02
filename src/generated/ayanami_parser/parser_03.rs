use super::*;

impl Parser {
    pub fn pvariant(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Variant");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("(")?;
            if let asuka::runtime::Value::Node(child) = self.ptype_list()? {
                n.set("type_list", asuka::runtime::Value::Node(child));
            }
            self.0.expect(")")?;
        } // end group
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("{")?;
            if let asuka::runtime::Value::Node(child) = self.pfield_list()? {
                n.set("field_list", asuka::runtime::Value::Node(child));
            }
            self.0.expect("}")?;
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pvariant_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("VariantList");
        if let asuka::runtime::Value::Node(child) = self.pvariant()? {
            n.set("variant", asuka::runtime::Value::Node(child));
        }
        loop {
            let _gr_saved = self.0.pos;
            if let Ok(_) = (|| -> Result<(), String> {
                    { // group
                    let _g_saved = self.0.pos;
                        self.0.expect(",")?;
                        if let asuka::runtime::Value::Node(child) = self.pvariant()? {
                            n.set("variant", asuka::runtime::Value::Node(child));
                        }
                    } // end group
                Ok(())
        })() {}
            else { self.0.pos = _gr_saved; break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pinterface_def(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("InterfaceDef");
        if let asuka::runtime::Value::Node(child) = self.pattr_list()? {
            n.set("attr_list", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pvis()? {
            n.set("vis", asuka::runtime::Value::Node(child));
        }
        self.0.expect("INTERFACE")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pgeneric_params()? {
            n.set("generic_params", asuka::runtime::Value::Node(child));
        }
        self.0.expect("{")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("iface_method_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect("}")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn piface_method(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("IfaceMethod");
        if let asuka::runtime::Value::Node(child) = self.pattr_list()? {
            n.set("attr_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect("FN")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
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
        self.0.expect(";")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pself_param(&mut self) -> Result<asuka::runtime::Value, String> {
        if self.0.tok().kind == "SELF" {
            let mut node = asuka::runtime::Node::new("self");
            self.0.expect("SELF")?;
            return Ok(asuka::runtime::Value::Node(Box::new(node)));
        }
        return Err(format!("no alt"));
    }

    pub fn pimpl_block(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ImplBlock");
        if let asuka::runtime::Value::Node(child) = self.pattr_list()? {
            n.set("attr_list", asuka::runtime::Value::Node(child));
        }
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
        if let asuka::runtime::Value::Node(child) = self.pattr_list()? {
            n.set("attr_list", asuka::runtime::Value::Node(child));
        }
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
}
