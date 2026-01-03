use std::assert;

#[derive(PartialEq)]
#[derive(Debug)]
#[derive(Clone)]
pub enum Token<'a> {
    Ident(&'a str),
    Symbol(char),
    StringLiteral(String),
    NumericLiteral(i64),
}

/// A lexer for a given text file. Produces a stream of Tokens
pub struct Lexer<'a> {
    data : &'a str,
    offset : usize,
    line: usize,
    buffered_token : Option<Token<'a>>,
}

impl<'a> Lexer<'a> {
    /// Create a lexer from an in memory string slice.
    pub fn from_str(data: &'a str) -> Lexer<'a> {
        Lexer {
            data,
            offset: 0,
            line: 1,
            buffered_token: None
        }
    }

    /// Return a token to the stream.
    pub fn unget_token(&mut self, token: Token<'a>) {
        assert!(self.buffered_token.is_none());
        self.buffered_token = Some(token);
    }

    /// Peek at the next token in the stream. Can only be used once between token reads.
    pub fn peek_token(&mut self) -> Option<Token<'a>> {
        assert!(self.buffered_token.is_none());
        let tok = self.next_token();
        if tok.is_some() {
            self.unget_token(tok.clone().unwrap());
        }
        tok.clone()
    }

    /// Peek at (return but do not consume) the next character in the stream.
    fn peek_char(&self) -> Option<char> {
        self.data[self.offset..].chars().next()
    }

    /// Consume the next character in the stream.
    fn eat_char(&mut self) {
        let c = self.peek_char().unwrap();
        self.offset += c.len_utf8();
    }

    /// Consume until the next non-whitespace character.
    fn eat_whitespace(&mut self) {
        loop {
            let c = self.peek_char();
            if c.is_none() {
                break;
            }
            if !c.unwrap().is_whitespace() {
                break;
            }
            self.eat_char();
        }
    }

    /// Return the next token, if any.
    pub fn next_token(&mut self) -> Option<Token<'a>> {
        self.eat_whitespace();
        let start_offset = self.offset;
        loop {
            let opt_c = self.peek_char();
            match opt_c {
                None => {
                    break;
                }
                Some(c) => {
                    if c == '"' {
                        // Start of a string literal.
                        let mut str_val = String::new();
                        // Eat the opening quote.
                        self.eat_char();
                        loop {
                            let str_c = self.peek_char();
                            if str_c.is_none() {
                                panic!("Unexpected end of file (missing '\"') on line {}", self.line);
                            }
                            self.eat_char();
                            if str_c.unwrap() == '\"' {
                                break;
                            }
                            str_val.push(str_c.unwrap());
                        }
                        return Some(Token::StringLiteral(str_val));
                    } else if c.is_numeric() || c == '-' {
                        // Start of a numeric (integer) literal.
                        self.eat_char();
                        loop {
                            let int_c = self.peek_char();
                            if int_c.is_none() || !int_c.unwrap().is_numeric() {
                                break;
                            }
                            self.eat_char();
                        }
                        let int_slice = &self.data[start_offset..self.offset];
                        let int_val = int_slice.parse::<i64>().unwrap();
                        return Some(Token::NumericLiteral(int_val));
                    } else if c.is_whitespace() {
                        if c == '\n' {
                            self.line += 1;
                        }
                        break;
                    } else if c == '/' {

                    } else if !c.is_alphanumeric() && c != '_' {
                        if self.offset != start_offset {
                            break;
                        }
                        self.eat_char();
                        return Some(Token::Symbol(c));
                    } else {
                        self.eat_char();
                    }
                }
            }
        }
        let end_offset = self.offset;
        if start_offset == end_offset {
            return None;
        }
        Some(Token::Ident(&self.data[start_offset..end_offset]))
    }

    /// Expect a specific 'ident' token, and panic if not available.
    pub fn expect_ident(&mut self, ident: &str) {
        let line = self.line;
        let tok = self.next_token();
        if tok.is_none() {
            panic!("Expected {} on line {}, but got EOF!", ident, line);
        }
        let tok_value = tok.unwrap();

        if tok_value != Token::Ident(ident) {
            panic!("Expected {} on line {}, but got {:?}!", ident, line, tok_value);
        }
    }

    /// Peek and see if the next token is a given ident.
    /// Note: requires &mut self because peeking is implemented as a mutation, even if it logically isn't one.
    pub fn is_next_ident(&mut self, ident: &str) -> bool {
        let tok = self.peek_token();
        if tok.is_none() {
            return false;
        }

        match tok.unwrap() {
            Token::Ident(val) => val == ident,
            _ => false
        }
    }

    /// Expect a specific symbol, and panic if not available.
    pub fn expect_symbol(&mut self, sym: char) {
        let line = self.line;
        let tok = self.next_token();
        if tok.is_none() {
            panic!("Expected '{}' on line {}, but got EOF!", sym, line);
        }
        let tok_value = tok.unwrap();

        if tok_value != Token::Symbol(sym) {
            panic!("Expected '{}' on line {}, but got {:?}!", sym, line, tok_value);
        }
    }

    /// Expect that the next token is a string, and return it, or panic if it isn't.
    pub fn get_string_literal(&mut self) -> String {
        let line = self.line;
        let tok = self.next_token();
        if tok.is_none() {
            panic!("Expected string literal on line {}, but got EOF!", line);
        }
        let tok_value = tok.unwrap();
        if let Token::StringLiteral(str_val) = tok_value {
            return str_val;
        } else {
            panic!("Expected string on line {}, but got {:?}!", line, tok_value);
        }
    }

    /// Expect that the next token is an integer literal, and return it. Or panic if it isn't.'
    pub fn get_int_literal(&mut self) -> i64 {
        let line = self.line;
        let tok = self.next_token();
        if tok.is_none() {
            panic!("Expected integer literal on line {}, but got EOF!", line);
        }
        let tok_value = tok.unwrap();
        if let Token::NumericLiteral(int_val) = tok_value {
            return int_val;
        } else {
            panic!("Expected integer literal on line {}, but got {:?}!", line, tok_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lexer_hello() {
        let hello_world = "Hello World";
        let mut lexer = Lexer::from_str(hello_world);
        let first_token = lexer.next_token().unwrap();
        assert_eq!(first_token, Token::Ident("Hello"));
        let second_token = lexer.next_token().unwrap();
        assert_eq!(second_token, Token::Ident("World"));


        assert!(lexer.next_token().is_none());
    }
    #[test]
    fn lexer_string_literal() {
        let input = "  \" This is a string \" ";
        let mut lexer = Lexer::from_str(input);
        let token = lexer.next_token().unwrap();
        assert_eq!(token, Token::StringLiteral(" This is a string ".to_string()));
        assert!(lexer.next_token().is_none());
    }
    #[test]
    fn lexer_script() {
        let test_input = "Filename=\"test.txt\"";
        let mut lexer = Lexer::from_str(test_input);
        assert_eq!(lexer.next_token().unwrap(), Token::Ident("Filename"));
        assert_eq!(lexer.next_token().unwrap(), Token::Symbol('='));
        assert_eq!(lexer.next_token().unwrap(), Token::StringLiteral("test.txt".to_string()));
        assert!(lexer.next_token().is_none());
    }
    #[test]
    fn lexer_script_with_ws() {
        let test_input = " Filename  =\n \"test.txt\"\n\n";
        let mut lexer = Lexer::from_str(test_input);
        assert_eq!(lexer.next_token().unwrap(), Token::Ident("Filename"));
        assert_eq!(lexer.next_token().unwrap(), Token::Symbol('='));
        assert_eq!(lexer.next_token().unwrap(), Token::StringLiteral("test.txt".to_string()));
        assert!(lexer.next_token().is_none());
    }
}
