use super::*;

impl Parser {
    pub fn pextern_c(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ExternC");
        self.0.expect("EXTERN")?;
        if let asuka::runtime::Value::Node(child) = self.ps()? {
            n.set("string_literal", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pstruct_def(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("StructDef");
        if let asuka::runtime::Value::Node(child) = self.pvis()? {
            n.set("vis", asuka::runtime::Value::Node(child));
        }
        self.0.expect("STRUCT")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pgeneric_params()? {
            n.set("generic_params", asuka::runtime::Value::Node(child));
        }
        self.0.expect("{")?;
        if let asuka::runtime::Value::Node(child) = self.pfield_list()? {
            n.set("field_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect("}")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfield(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Field");
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfield_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FieldList");
        loop {
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pfield()? {
                n.set("field", asuka::runtime::Value::Node(child));
            } else { break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn penum_def(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("EnumDef");
        if let asuka::runtime::Value::Node(child) = self.pvis()? {
            n.set("vis", asuka::runtime::Value::Node(child));
        }
        self.0.expect("ENUM")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pgeneric_params()? {
            n.set("generic_params", asuka::runtime::Value::Node(child));
        }
        self.0.expect("{")?;
        if let asuka::runtime::Value::Node(child) = self.pvariant_list()? {
            n.set("variant_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect("}")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

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
}
