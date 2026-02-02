use crate::{error_type::Error, tokenlizer::Tokenlizer, types::Token};

impl Tokenlizer {
    fn is_alpha(c: char) -> bool {
        c.is_alphabetic()
    }
    fn is_num(c: char) -> bool {
        c.is_ascii_digit() || c == '.'
    }
    fn is_operator(c: char) -> bool {
        c == '+'
            || c == '-'
            || c == '*'
            || c == '/'
            || c == '='
            || c == '!'
            || c == '<'
            || c == '>'
            || c == '{'
            || c == '}'
            || c == '('
            || c == ')'
            || c == '['
            || c == ']'
            || c == '\''
            || c == '\\'
            || c == '\"'
            || c == ','
    }

    fn is_keyword(s: String) -> bool {
        s == "int"
            || s == "float"
            || s == "char"
            || s == "bool"
            || s == "const"
            || s == "for"
            || s == "in"
            || s == "while"
            || s == "fn"
            || s == "ref"
            || s == "return"
            || s == "if"
            || s == "elif"
            || s == "else"
    }

    fn tokenlize_identifier(&mut self) -> Token {
        let mut _s = String::new();

        while self.i < self.chars.len() && Self::is_alpha(self.chars[self.i]) {
            _s.push(self.chars[self.i]);
            self.i += 1;
        }

        if Self::is_keyword(_s.clone()) {
            Token::Keyword(_s)
        } else {
            Token::Identifier(_s)
        }
    }
    fn tokenlize_operator(&mut self) -> Result<Token, Error> {
        self.i += 1;
        Ok(Token::Operator(self.chars[self.i - 1].to_string()))
    }
    fn tokenlize_num(&mut self) -> Result<Token, Error> {
        let index = self.i;
        let mut _s = String::new();

        while self.i < self.chars.len() && Self::is_num(self.chars[self.i]) {
            _s.push(self.chars[self.i]);
            self.i += 1;
        }

        match _s.parse::<f64>() {
            Ok(x) => Ok(Token::Num(x)),
            Err(_) => Err(Error::new_error(format!(
                "{} cannot parse from string {}",
                index, _s
            ))),
        }
    }

    pub(crate) fn tokenlize(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::<Token>::new();

        while self.i < self.chars.len() {
            // special-case string literal start: split opening quote, content, closing quote
            if self.chars[self.i] == '"' {
                // opening quote token
                tokens.push(Token::Operator("\"".to_string()));
                self.i += 1;

                // collect string content (allow any characters, handle simple escapes)
                let mut s = String::new();
                while self.i < self.chars.len() && self.chars[self.i] != '"' {
                    let c = self.chars[self.i];
                    if c == '\\' {
                        // escape sequence: take next char literally and translate common escapes
                        self.i += 1;
                        if self.i >= self.chars.len() {
                            return Err(Error::new_error(
                                "unterminated escape in string".to_string(),
                            ));
                        }
                        let esc = self.chars[self.i];
                        let mapped = match esc {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            '\\' => '\\',
                            '"' => '"',
                            '\'' => '\'',
                            '0' => '\0',
                            other => other,
                        };
                        s.push(mapped);
                        self.i += 1;
                    } else {
                        s.push(c);
                        self.i += 1;
                    }
                }

                // push content as Identifier token (parser will accept Identifier/Operator/Num inside string)
                tokens.push(Token::Identifier(s));

                // expect and emit closing quote
                if self.i < self.chars.len() && self.chars[self.i] == '"' {
                    tokens.push(Token::Operator("\"".to_string()));
                    self.i += 1;
                } else {
                    return Err(Error::new_error("unterminated string literal".to_string()));
                }

                continue;
            }
            if self.chars[self.i] == '\'' {
                // opening quote token
                tokens.push(Token::Operator("\'".to_string()));
                self.i += 1;

                // collect string content (allow any characters, handle simple escapes)
                let mut s = String::new();
                while self.i < self.chars.len() && self.chars[self.i] != '\'' {
                    let c = self.chars[self.i];
                    if c == '\\' {
                        // escape sequence: take next char literally and translate common escapes
                        self.i += 1;
                        if self.i >= self.chars.len() {
                            return Err(Error::new_error(
                                "unterminated escape in string".to_string(),
                            ));
                        }
                        let esc = self.chars[self.i];
                        let mapped = match esc {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            '\\' => '\\',
                            '"' => '"',
                            '\'' => '\'',
                            '0' => '\0',
                            other => other,
                        };
                        s.push(mapped);
                        self.i += 1;
                    } else {
                        s.push(c);
                        self.i += 1;
                    }
                }

                // push content as Identifier token (parser will accept Identifier/Operator/Num inside string)
                tokens.push(Token::Identifier(s));

                // expect and emit closing quote
                if self.i < self.chars.len() && self.chars[self.i] == '\'' {
                    tokens.push(Token::Operator("\'".to_string()));
                    self.i += 1;
                } else {
                    return Err(Error::new_error("unterminated string literal".to_string()));
                }

                continue;
            }
            
            if Self::is_num(self.chars[self.i]) {
                match self.tokenlize_num() {
                    Ok(x) => tokens.push(x),
                    Err(mut e) => return Err(e.with_context_front(format!(""))),
                };
            } else if Self::is_alpha(self.chars[self.i]) {
                tokens.push(self.tokenlize_identifier());
            } else if Self::is_operator(self.chars[self.i]) {
                match self.tokenlize_operator() {
                    Ok(x) => tokens.push(x),
                    Err(mut e) => return Err(e.with_context_front(format!(""))),
                };
            } else if self.chars[self.i].is_whitespace() {
                // skip spaces/tabs/newlines
                self.i += 1;
            } else {
                return Err(Error::new_error(format!(
                    "unknown token: {}",
                    self.chars[self.i]
                )));
            }
        }

        Ok(tokens)
    }

    pub(crate) fn new(s: String) -> Tokenlizer {
        let chars: Vec<char> = s.chars().collect();
        Tokenlizer { i: 0, chars: chars }
    }
}
