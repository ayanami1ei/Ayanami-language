// @generated
#[allow(unused)]

pub const KEYWORDS: &[&str] = &["fn", "return", "if", "else", "true", "false", "while", "for", "in", "match", "enum", "struct", "interface", "impl", "pub", "shared", "unique", "weak", "ref", "mut", "extern", "import", "as", "break", "continue", "self", "move", "clone", "inline", "let"];

pub fn tokenize(input: &str) -> Vec<asuka::runtime::Token> {
    let mut lex = asuka::runtime::Lexer::new(input);
    let mut tokens = Vec::new();
    loop {
        lex.skip_ws();
        if lex.pos >= lex.chars.len() { break; }
        let c = lex.chars[lex.pos];
        match c {
            '"' => tokens.push(lex.read_string()),
            c if c.is_ascii_digit() => tokens.push(lex.read_number()),
            c if c.is_alphabetic() || c == '_' => tokens.push(lex.read_ident(KEYWORDS)),
            '!' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='=' { tokens.push(lex.read_fixed("!=", "!=")); }
                else { tokens.push(lex.read_fixed("!", "!")); }
            }
            '%' => tokens.push(lex.read_fixed("%", "%")),
            '&' => tokens.push(lex.read_fixed("&&", "&&")),
            '(' => tokens.push(lex.read_fixed("(", "(")),
            ')' => tokens.push(lex.read_fixed(")", ")")),
            '*' => tokens.push(lex.read_fixed("*", "*")),
            '+' => tokens.push(lex.read_fixed("+", "+")),
            ',' => tokens.push(lex.read_fixed(",", ",")),
            '-' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='>' { tokens.push(lex.read_fixed("->", "->")); }
                else { tokens.push(lex.read_fixed("-", "-")); }
            }
            '.' => tokens.push(lex.read_fixed(".", ".")),
            '/' => tokens.push(lex.read_fixed("/", "/")),
            ':' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]==':' { tokens.push(lex.read_fixed("::", "::")); }
                else { tokens.push(lex.read_fixed(":", ":")); }
            }
            ';' => tokens.push(lex.read_fixed(";", ";")),
            '<' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='=' { tokens.push(lex.read_fixed("<=", "<=")); }
                else { tokens.push(lex.read_fixed("<", "<")); }
            }
            '=' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='=' { tokens.push(lex.read_fixed("==", "==")); }
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='>' { tokens.push(lex.read_fixed("=>", "=>")); }
                else { tokens.push(lex.read_fixed("=", "=")); }
            }
            '>' => {
                if lex.pos+1<lex.chars.len() && lex.chars[lex.pos+1]=='=' { tokens.push(lex.read_fixed(">=", ">=")); }
                else { tokens.push(lex.read_fixed(">", ">")); }
            }
            '[' => tokens.push(lex.read_fixed("[", "[")),
            ']' => tokens.push(lex.read_fixed("]", "]")),
            '{' => tokens.push(lex.read_fixed("{", "{")),
            '|' => tokens.push(lex.read_fixed("||", "||")),
            '}' => tokens.push(lex.read_fixed("}", "}")),
            _ => panic!("unexpected '{}'", c),
        }
    }
    tokens.push(asuka::runtime::Token { kind: "EOF".into(), span: asuka::runtime::Span::new(), value: String::new() });
    tokens
}

pub struct Parser(pub asuka::runtime::Parser);
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
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
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
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pvis(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Vis");
        self.0.expect("PUB")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pinline(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("Inline");
        self.0.expect("INLINE")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

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
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pvariant_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("VariantList");
        if let asuka::runtime::Value::Node(child) = self.pvariant()? {
            n.set("variant", asuka::runtime::Value::Node(child));
        }
        loop {
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
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
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        self.0.expect(")")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        self.0.expect(";")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pself_param(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("SelfParam");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
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
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        self.0.expect(")")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
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
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
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
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
        }
        self.0.expect("]")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pgeneric_param(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("GenericParam");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
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
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

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
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn parray_type(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ArrayType");
        self.0.expect("[")?;
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        self.0.expect("]")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn ptype_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("TypeList");
        if let asuka::runtime::Value::Node(child) = self.ptyp()? {
            n.set("typ", asuka::runtime::Value::Node(child));
        }
        loop {
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
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

    pub fn pindex_assign(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("IndexAssign");
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect("[")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect("]")?;
        self.0.expect("=")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect(";")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn preturn_stmt(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ReturnStmt");
        self.0.expect("RETURN")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect(";")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pif_stmt(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("IfStmt");
        self.0.expect("IF")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pblock()? {
            n.set("block", asuka::runtime::Value::Node(child));
        }
        loop {
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pwhile_stmt(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("WhileStmt");
        self.0.expect("WHILE")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pblock()? {
            n.set("block", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfor_stmt(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("ForStmt");
        self.0.expect("FOR")?;
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("ident", asuka::runtime::Value::Node(child));
        }
        self.0.expect("IN")?;
        self.0.expect("(")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        self.0.expect(",")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        self.0.expect(")")?;
        if let asuka::runtime::Value::Node(child) = self.pblock()? {
            n.set("block", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

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
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn ppattern_args(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("PatternArgs");
        if let asuka::runtime::Value::Node(child) = self.ppattern()? {
            n.set("pattern", asuka::runtime::Value::Node(child));
        }
        loop {
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
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
        if let Ok(val) = self.pto_shared_expr() { return Ok(val); }
        if let Ok(val) = self.pto_weak_expr() { return Ok(val); }
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

    fn parse_expr_prec(&mut self, min_prec: u32) -> Result<asuka::runtime::Value, String> {
        let mut lhs = self.pexpr_primary()?;
        loop {
            let tok = self.0.tok().clone();
            let prec = match tok.kind.as_str() {
                "+" => Some((5u32, 6u32)),
                "-" => Some((5u32, 6u32)),
                "*" => Some((6u32, 7u32)),
                "/" => Some((6u32, 7u32)),
                "%" => Some((6u32, 7u32)),
                "==" => Some((3u32, 4u32)),
                "!=" => Some((3u32, 4u32)),
                "<" => Some((3u32, 4u32)),
                ">" => Some((3u32, 4u32)),
                "<=" => Some((3u32, 4u32)),
                ">=" => Some((3u32, 4u32)),
                "&&" => Some((4u32, 5u32)),
                "||" => Some((4u32, 5u32)),
                "=" => Some((2u32, 2u32)),
                "!" => Some((1u32, 2u32)),
                "-" => Some((7u32, 8u32)),
                "[" => Some((1u32, 2u32)),
                "]" => Some((1u32, 2u32)),
                _ => None,
            };
            match prec {
                Some((this_prec, next_prec)) if this_prec >= min_prec => {
                    self.0.adv(); // consume operator
                    let rhs = self.parse_expr_prec(next_prec)?;
                    let mut node = asuka::runtime::Node::new("BinaryExpr");
                    node.set("lhs", lhs);
                    node.set("op", asuka::runtime::Value::String(tok.kind));
                    node.set("rhs", rhs);
                    lhs = asuka::runtime::Value::Node(Box::new(node));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    fn pexpr_primary(&mut self) -> Result<asuka::runtime::Value, String> {
        // Try all expression alternatives except BinaryExpr
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
        if let Ok(val) = self.pto_shared_expr() { return Ok(val); }
        if let Ok(val) = self.pto_weak_expr() { return Ok(val); }
        if let Ok(val) = self.pref_expr() { return Ok(val); }
        if let Ok(val) = self.ptry_op() { return Ok(val); }
        if let Ok(val) = self.pstruct_literal() { return Ok(val); }
        if let Ok(val) = self.parray_literal() { return Ok(val); }
        if let Ok(val) = self.parray_sized() { return Ok(val); }
        if let Ok(val) = self.penum_construct() { return Ok(val); }
        if let Ok(val) = self.pasm_expr() { return Ok(val); }
        if let Ok(val) = self.pnull_expr() { return Ok(val); }
        if let Ok(val) = self.pstruct_literal() { return Ok(val); }
        if let Ok(val) = self.pi() { return Ok(val); }
        Err("expected expression".into())
    }

    pub fn punary_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("UnaryExpr");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

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
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
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
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
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
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
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
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
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
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pasm_expr(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("AsmExpr");
        self.0.expect("ASM")?;
        self.0.expect("(")?;
        if let asuka::runtime::Value::Node(child) = self.ps()? {
            n.set("string_literal", asuka::runtime::Value::Node(child));
        }
        self.0.expect(":")?;
        if let asuka::runtime::Value::Node(child) = self.pasm_output_list()? {
            n.set("asm_output_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect(":")?;
        if let asuka::runtime::Value::Node(child) = self.pasm_input_list()? {
            n.set("asm_input_list", asuka::runtime::Value::Node(child));
        }
        self.0.expect(")")?;
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pasm_output(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("AsmOutput");
        if let asuka::runtime::Value::Node(child) = self.ps()? {
            n.set("string_literal", asuka::runtime::Value::Node(child));
        }
        self.0.expect(":")?;
        if let asuka::runtime::Value::Node(child) = self.pexpr()? {
            n.set("expr", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pasm_output_list(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("AsmOutputList");
        if let asuka::runtime::Value::Node(child) = self.pasm_output()? {
            n.set("asm_output", asuka::runtime::Value::Node(child));
        }
        loop {
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
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
            let saved = self.0.pos;
            match self.0.tok() {
                asuka::runtime::Token { kind, .. } if matches!(kind.as_str(), "EOF" | "}" | ";") => break,
                _ => {}
            }
            if let asuka::runtime::Value::Node(child) = self.pi()? {
                n.set("", asuka::runtime::Value::Node(child));
            } else { break; }
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
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pfloat_literal(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("FloatLiteral");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("token", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pstring_literal(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("StringLiteral");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("token", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

    pub fn pchar_literal(&mut self) -> Result<asuka::runtime::Value, String> {
        let mut n = asuka::runtime::Node::new("CharLiteral");
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("token", asuka::runtime::Value::Node(child));
        }
        if let asuka::runtime::Value::Node(child) = self.pi()? {
            n.set("", asuka::runtime::Value::Node(child));
        }
        Ok(asuka::runtime::Value::Node(Box::new(n)))
    }

}

