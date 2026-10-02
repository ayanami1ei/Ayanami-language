use super::*;

impl<'a> Lexer<'a> {
    pub(super) fn is_ident_start(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || (c as u32) >= 0x80
    }
    pub(super) fn is_ident_continue(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || (c as u32) >= 0x80
    }

    pub(super) fn read_identifier_or_keyword(&mut self) -> (String, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
        let mut s = String::new();
        if let Some(c) = self.peek()
            && Self::is_ident_start(c)
        {
            s.push(self.bump().unwrap());
        }
        while let Some(c) = self.peek() {
            if Self::is_ident_continue(c) {
                s.push(self.bump().unwrap());
            } else {
                break;
            }
        }
        (s, start_line, start_col, start_byte)
    }

    pub(super) fn read_number(&mut self) -> (String, bool, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
        let mut s = String::new();
        let mut is_float = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(self.bump().unwrap());
            } else {
                break;
            }
        }
        if let Some('.') = self.peek()
            && let Some(nxt) = self.peek_next()
            && nxt.is_ascii_digit()
        {
            is_float = true;
            s.push(self.bump().unwrap());
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    s.push(self.bump().unwrap());
                } else {
                    break;
                }
            }
        }
        (s, is_float, start_line, start_col, start_byte)
    }

    pub(super) fn read_char_literal(&mut self) -> (String, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
        // consume opening '
        self.bump();
        let mut s = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\\' => {
                    s.push(self.bump().unwrap());
                    if let Some(_n) = self.peek() {
                        s.push(self.bump().unwrap());
                    }
                }
                '\'' => {
                    self.bump();
                    break;
                }
                _ => {
                    s.push(self.bump().unwrap());
                }
            }
        }
        (s, start_line, start_col, start_byte)
    }

    pub(super) fn read_string_literal(&mut self) -> (String, usize, usize, usize) {
        let start_line = self.line;
        let start_col = self.col;
        let start_byte = self.byte_offset;
        self.bump(); // consume '"'
        let mut s = String::new();
        while let Some(c) = self.peek() {
            match c {
                '\\' => {
                    s.push(self.bump().unwrap());
                    if let Some(_n) = self.peek() {
                        s.push(self.bump().unwrap());
                    }
                }
                '"' => {
                    self.bump();
                    break;
                }
                _ => {
                    s.push(self.bump().unwrap());
                }
            }
        }
        (s, start_line, start_col, start_byte)
    }
}
