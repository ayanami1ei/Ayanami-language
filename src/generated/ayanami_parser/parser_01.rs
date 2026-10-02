use super::*;

impl Parser {
    pub fn new(tokens: Vec<asuka::runtime::Token>) -> Self { Self(asuka::runtime::Parser::new(tokens)) }

    pub fn pi(&mut self) -> Result<asuka::runtime::Value, String> {
        let t = self.0.tok().clone();
        if t.kind != "Ident" { return Err("expected ident".into()); }
        self.0.adv();
        let mut node = asuka::runtime::Node::new("Ident");
        node.set("value", asuka::runtime::Value::String(t.value));
        Ok(asuka::runtime::Value::Node(Box::new(node)))
    }
    pub fn pn(&mut self) -> Result<asuka::runtime::Value, String> {
        let t = self.0.tok().clone();
        if t.kind != "IntLit" { return Err("expected int".into()); }
        self.0.adv();
        let n: i64 = t.value.parse().map_err(|_| "bad int")?;
        let mut node = asuka::runtime::Node::new("IntLit");
        node.set("value", asuka::runtime::Value::Int(n));
        Ok(asuka::runtime::Value::Node(Box::new(node)))
    }
    pub fn ps(&mut self) -> Result<asuka::runtime::Value, String> {
        let t = self.0.tok().clone();
        if t.kind != "StrLit" { return Err("expected string".into()); }
        self.0.adv();
        let mut node = asuka::runtime::Node::new("StrLit");
        node.set("value", asuka::runtime::Value::String(t.value));
        Ok(asuka::runtime::Value::Node(Box::new(node)))
    }

    pub fn pprogram(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Program");
        loop {
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pitem()? {
                n.set("item", asuka::runtime::Value::Node(child));
            } else { break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pitem(&mut self) -> Result<asuka::runtime::Value, String> {
        if let Ok(val) = self.pfn_decl() { return Ok(val); }
        if let Ok(val) = self.pstruct_def() { return Ok(val); }
        if let Ok(val) = self.penum_def() { return Ok(val); }
        if let Ok(val) = self.pinterface_def() { return Ok(val); }
        if let Ok(val) = self.pimpl_block() { return Ok(val); }
        if let Ok(val) = self.pimport() { return Ok(val); }
        if let Ok(val) = self.pnamespace() { return Ok(val); }
        return Err(format!("no alt"));
    }

    pub fn pimport(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Import");
        self.0.expect("IMPORT")?;
        if let asuka::runtime::Value::Node(child) = self.ps()? {
            n.set("string_literal", asuka::runtime::Value::Node(child));
        }
        self.0.expect(";")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pnamespace(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Namespace");
        if let asuka::runtime::Value::Node(child) = self.pvis()? {
            n.set("vis", asuka::runtime::Value::Node(child));
        }
        self.0.expect("NAMESPACE")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        self.0.expect("{")?;
        loop {
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pitem()? {
                n.set("item", asuka::runtime::Value::Node(child));
            } else { break; }
        }
        self.0.expect("}")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfn_decl(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FnDecl");
        if let asuka::runtime::Value::Node(child) = self.pvis()? {
            n.set("vis", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pinline()? {
            n.set("inline", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pextern_c()? {
            n.set("extern_c", asuka::runtime::Value::Node(child));
        }
        self.0.expect("FN")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pgeneric_params()? {
            n.set("generic_params", asuka::runtime::Value::Node(child));
        }
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

    pub fn pfn_param(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FnParam");
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pparam_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ParamList");
        if let asuka::runtime::Value::Node(child) = self.pfn_param()? {
            n.set("fn_param", asuka::runtime::Value::Node(child));
        }
        loop {
            let _gr_saved = self.0.pos;
            if let Ok(_) = (|| -> Result<(), String> {
                    { // group
                    let _g_saved = self.0.pos;
                        self.0.expect(",")?;
                        if let asuka::runtime::Value::Node(child) = self.pfn_param()? {
                            n.set("fn_param", asuka::runtime::Value::Node(child));
                        }
                    } // end group
                Ok(())
        })() {}
            else { self.0.pos = _gr_saved; break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pvis(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Vis");
        self.0.expect("PUB")?;
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("(")?;
            self.0.expect("CRATE")?;
            self.0.expect(")")?;
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pinline(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Inline");
        self.0.expect("INLINE")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }
}
