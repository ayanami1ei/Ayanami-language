use anyhow::{Error, anyhow};

use crate::{
    intern::Symbol,
    lexer::{Delimiter, Keyword, Token, TokenKind},
    parser::{
        ast::{Program, Type},
        symbol::{
            fn_symbol::FnSymbol,
            semantic_symbol::SemanticSymbol,
            tree::Tree,
            var_symbol::VarSymbol,
        },
    },
    span::Span,
};

#[allow(dead_code)]
pub struct Parser {
    map: Tree<Vec<SemanticSymbol>>,
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser {
    pub fn new() -> Self {
        Self { map: Tree::new() }
    }

    fn parse_type(&self, token: &Token) -> Type {
        let span = Span::new(token.line, token.col, token.line, token.col);
        match &token.kind {
            TokenKind::Keyword(Keyword::Int) => Type::Int(span),
            TokenKind::Keyword(Keyword::Float) => Type::Float(span),
            TokenKind::Keyword(Keyword::Char) => Type::Char(span),
            TokenKind::Identifier(name) => Type::Named(Symbol::intern(name), span),
            _ => Type::Void(span),
        }
    }

    fn parse_symbol(&mut self, tokens: &[Token]) -> Result<(), Error> {
        let mut i = 0;
        let mut pending_params: Vec<VarSymbol> = Vec::new();

        while i < tokens.len() {
            let token = &tokens[i];

            match &token.kind {
                TokenKind::Keyword(Keyword::Fn) => {
                    let span = Span::new(token.line, token.col, token.line, token.col);

                    let name = match tokens.get(i + 1) {
                        Some(Token {
                            kind: TokenKind::Identifier(name_str),
                            ..
                        }) => Symbol::intern(name_str),
                        _ => {
                            return Err(anyhow!(
                                "{}, expected function name after `fn`",
                                span
                            ))
                        }
                    };

                    // 找到 ) 结束参数列表
                    let mut end = i + 2;
                    while end < tokens.len()
                        && !matches!(
                            &tokens[end].kind,
                            TokenKind::Delimiter(Delimiter::RParen)
                        )
                    {
                        end += 1;
                    }

                    // 解析形参
                    let mut params = Vec::<(Symbol, Type)>::new();
                    let mut k = i + 3; // 跳过 fn、名字、(
                    while k < end {
                        if let TokenKind::Identifier(param_name) = &tokens[k].kind {
                            let pname = Symbol::intern(param_name);
                            k += 1;
                            // 跳过冒号
                            if k < end
                                && matches!(&tokens[k].kind, TokenKind::Operator(op) if op == ":")
                            {
                                k += 1;
                                if k < end {
                                    let ptype = self.parse_type(&tokens[k]);
                                    params.push((pname, ptype.clone()));
                                }
                            }
                        }
                        k += 1;
                    }

                    // 解析返回类型
                    let rtn_type =
                        if end + 1 < tokens.len()
                            && matches!(
                                &tokens[end + 1].kind,
                                TokenKind::Delimiter(Delimiter::Arrow)
                            )
                        {
                            tokens
                                .get(end + 2)
                                .map(|t| self.parse_type(t))
                                .unwrap_or_else(|| Type::Void(Span::default()))
                        } else {
                            Type::Void(Span::default())
                        };

                    let param_symbols: Vec<VarSymbol> = params
                        .iter()
                        .cloned()
                        .map(|(n, t)| VarSymbol { name: n, type_: t })
                        .collect();

                    // Fn 符号放入外层作用域
                    self.map.push_to_current(SemanticSymbol::Fn(FnSymbol {
                        name,
                        rtn_type,
                        args: param_symbols.clone(),
                    }));

                    // 形参等进入函数体作用域后推入
                    pending_params = param_symbols;
                }
                TokenKind::Delimiter(delimiter) => match delimiter {
                    Delimiter::LBrace => {
                        self.map.into_next();
                        for param in pending_params.drain(..) {
                            self.map
                                .push_to_current(SemanticSymbol::Var(param));
                        }
                    }
                    Delimiter::RBrace => {
                        self.map.return_to_upper();
                    }
                    _ => {}
                },
                TokenKind::Identifier(name_str) => {
                    // 变量赋值：name = expr
                    if let Some(next) = tokens.get(i + 1) {
                        if matches!(&next.kind, TokenKind::Operator(op) if op == "=") {
                            let var_sym = VarSymbol {
                                name: Symbol::intern(name_str),
                                type_: Type::Void(Span::default()),
                            };
                            self.map
                                .push_to_current(SemanticSymbol::Var(var_sym));
                        }
                    }
                }
                _ => {}
            }

            i += 1;
        }

        Ok(())
    }

    pub fn parse(&mut self, tokens: &[Token]) -> Result<Program, Error> {
        self.parse_symbol(tokens)?;
        Ok(Program::new(Vec::new()))
    }
}
