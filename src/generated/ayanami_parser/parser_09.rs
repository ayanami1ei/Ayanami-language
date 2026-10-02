use super::*;

impl Parser {
    pub fn pasm_output_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("AsmOutputList");
        if let asuka::runtime::Value::Node(child) = self.pasm_output()? {
            n.set("asm_output", asuka::runtime::Value::Node(child));
        }
        loop {
            let _gr_saved = self.0.pos;
            if let Ok(_) = (|| -> Result<(), String> {
                    { // group
                    let _g_saved = self.0.pos;
                        self.0.expect(",")?;
                        if let asuka::runtime::Value::Node(child) = self.pasm_output()? {
                            n.set("asm_output", asuka::runtime::Value::Node(child));
                        }
                    } // end group
                Ok(())
        })() {}
            else { self.0.pos = _gr_saved; break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pasm_input(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("AsmInput");
        if let asuka::runtime::Value::Node(child) = self.ps()? {
            n.set("string_literal", asuka::runtime::Value::Node(child));
        }
        self.0.expect(":")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pasm_input_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("AsmInputList");
        if let asuka::runtime::Value::Node(child) = self.pasm_input()? {
            n.set("asm_input", asuka::runtime::Value::Node(child));
        }
        loop {
            let _gr_saved = self.0.pos;
            if let Ok(_) = (|| -> Result<(), String> {
                    { // group
                    let _g_saved = self.0.pos;
                        self.0.expect(",")?;
                        if let asuka::runtime::Value::Node(child) = self.pasm_input()? {
                            n.set("asm_input", asuka::runtime::Value::Node(child));
                        }
                    } // end group
                Ok(())
        })() {}
            else { self.0.pos = _gr_saved; break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pbool_literal(&mut self) -> Result<asuka::runtime::Value, String> {
        if self.0.tok().kind == "TRUE" {
            let mut node = asuka::runtime::Node::new("true");
            self.0.expect("TRUE")?;
            return Ok(asuka::runtime::Value::Node(Box::new(node)));
        }
        if self.0.tok().kind == "FALSE" {
            let mut node = asuka::runtime::Node::new("false");
            self.0.expect("FALSE")?;
            return Ok(asuka::runtime::Value::Node(Box::new(node)));
        }
        return Err(format!("no alt"));
    }

    pub fn pint_literal(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("IntLiteral");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("token", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("INTLITERAL")?;
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfloat_literal(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FloatLiteral");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("token", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("FLOATLITERAL")?;
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pstring_literal(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("StringLiteral");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("token", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("STRLIT")?;
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pchar_literal(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("CharLiteral");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("token", asuka::runtime::Value::Node(child));
        }
        { // group
        let _g_saved = self.0.pos;
            self.0.expect("CHARLITERAL")?;
        } // end group
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }
}
