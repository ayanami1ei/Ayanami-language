use crate::{error_type::Error, tokenlizer::Tokenlizer, types::Token};

impl Tokenlizer {
    fn is_alpha(c: char) -> bool {
        c.is_alphabetic()
    }
    fn is_num(c: char) -> bool {
        c.is_ascii_digit() || c=='.'
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
            } else if self.chars[self.i] == ' ' || self.chars[self.i] == '\n' {
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
