use crate::types::Token;

pub struct Lexer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_byte(&mut self) -> Option<u8> {
        if self.pos >= self.data.len() {
            None
        } else {
            let curr = self.data[self.pos];
            self.pos += 1;
            Some(curr)
        }
    }

    fn unread_byte(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }

    fn read_literal(&mut self) -> Option<Token> {
        let start = self.pos - 1;
        loop {
            match self.read_byte() {
                Some(val) if val.is_ascii_alphabetic() => {}
                x => {
                    if x.is_some() {
                        self.unread_byte();
                    }
                    let lit = &self.data[start..self.pos];
                    return Lexer::read_kw(lit)
                        .or_else(|| Some(Token::Ident(String::from_utf8_lossy(lit).to_string())));
                }
            }
        }
    }

    fn read_kw(kw: &[u8]) -> Option<Token> {
        match kw {
            b"fn" => Some(Token::FunctionDef),
            b"str" => Some(Token::StrType),
            b"float" => Some(Token::FloatType),
            b"int" => Some(Token::IntType),
            b"bool" => Some(Token::BoolType),
            b"true" => Some(Token::Bool(true)),
            b"false" => Some(Token::Bool(false)),
            _ => None,
        }
    }

    fn read_string(&mut self) -> Option<Token> {
        // todo: allow escaping chars
        let start = self.pos;
        loop {
            let b = self.read_byte()?;
            if b == b'"' {
                return Some(Token::String(
                    String::from_utf8_lossy(&self.data[start..self.pos - 1]).to_string(),
                ));
            }
        }
    }

    fn read_number(&mut self) -> Option<Token> {
        let start = self.pos - 1;
        loop {
            match self.read_byte() {
                Some(x) if x.is_ascii_digit() || x == b'.' => {}
                x => {
                    if x.is_some() {
                        self.unread_byte();
                    }
                    let num = &self.data[start..self.pos];
                    if num.contains(&b'.') {
                        let f: f64 = str::from_utf8(num).ok()?.parse().ok()?;
                        return Some(Token::Float(f));
                    } else {
                        let n: i64 = str::from_utf8(num).ok()?.parse().ok()?;
                        return Some(Token::Int(n));
                    }
                }
            }
        }
    }

    fn skip_line(&mut self) {
        loop {
            match self.read_byte() {
                None => return,
                Some(b'\n') => {
                    return;
                }
                _ => {}
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.read_byte() {
                None => return,
                Some(b) if b.is_ascii_whitespace() => {}
                Some(b'/') => match self.read_byte() {
                    Some(b'/') => self.skip_line(),
                    _ => {
                        self.unread_byte();
                        self.unread_byte();
                        return;
                    }
                },
                _ => {
                    self.unread_byte();
                    return;
                }
            }
        }
    }

    pub fn read(&mut self) -> Option<Token> {
        self.skip_whitespace_and_comments();

        match self.read_byte()? {
            b';' => Some(Token::Semicolon),
            b'(' => Some(Token::LParen),
            b')' => Some(Token::RParen),
            b'{' => Some(Token::LBrace),
            b'}' => Some(Token::RBrace),
            b':' => match self.read_byte() {
                Some(b'=') => Some(Token::Assign),
                _ => {
                    self.unread_byte();
                    Some(Token::Colon)
                }
            },
            b'"' => self.read_string(),
            x if x.is_ascii_alphabetic() => self.read_literal(),
            x if x.is_ascii_digit() || x == b'.' || x == b'+' || x == b'-' => self.read_number(),
            x => todo!("unexpected char `{}`", x as char),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_token_hello_world() {
        let input = r#"fn main() {
            println("Hello world");
        }
"#;
        let mut l = Lexer::new(input.as_bytes());
        let expected = [
            Token::FunctionDef,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::Ident("println".to_string()),
            Token::LParen,
            Token::String("Hello world".to_string()),
            Token::RParen,
            Token::Semicolon,
            Token::RBrace,
        ];

        for e in expected {
            let t = l.read();
            assert_eq!(e, t.unwrap())
        }
    }

    #[test]
    fn test_next_token_vars_comments() {
        let input = r#"fn main() {
            a := "string"; // type str
            println(a);

            b := 12; // type int
            println(b);

            c := 1.1; // type float
            println(c);

            d := true; // type bool
            println(d);

            // type hint
            e str := "abc";
            println(e);

            // compiler error vvv; g is of type int
            // g float := 12;
            // println(g);
        }
"#;
        let mut l = Lexer::new(input.as_bytes());
        let expected = [
            Token::FunctionDef,
            Token::Ident("main".to_string()),
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::Ident("a".to_string()),
            Token::Assign,
            Token::String("string".to_string()),
            Token::Semicolon,
            Token::Ident("println".to_string()),
            Token::LParen,
            Token::Ident("a".to_string()),
            Token::RParen,
            Token::Semicolon,
            Token::Ident("b".to_string()),
            Token::Assign,
            Token::Int(12),
            Token::Semicolon,
            Token::Ident("println".to_string()),
            Token::LParen,
            Token::Ident("b".to_string()),
            Token::RParen,
            Token::Semicolon,
            Token::Ident("c".to_string()),
            Token::Assign,
            Token::Float(1.1),
            Token::Semicolon,
            Token::Ident("println".to_string()),
            Token::LParen,
            Token::Ident("c".to_string()),
            Token::RParen,
            Token::Semicolon,
            Token::Ident("d".to_string()),
            Token::Assign,
            Token::Bool(true),
            Token::Semicolon,
            Token::Ident("println".to_string()),
            Token::LParen,
            Token::Ident("d".to_string()),
            Token::RParen,
            Token::Semicolon,
            Token::Ident("e".to_string()),
            Token::StrType,
            Token::Assign,
            Token::String("abc".to_string()),
            Token::Semicolon,
            Token::Ident("println".to_string()),
            Token::LParen,
            Token::Ident("e".to_string()),
            Token::RParen,
            Token::Semicolon,
            Token::RBrace,
        ];

        for e in expected {
            let t = l.read();
            assert_eq!(e, t.unwrap())
        }
    }
}
