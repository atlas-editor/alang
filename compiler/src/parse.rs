use std::{collections::HashMap, iter::Peekable};

use crate::{
    lex::{Lexer, Token},
    parsingerr,
    types::ParsingError,
};

pub enum Operator {
    Plus,
}

pub enum Expression {
    Identifier(String),
    IdentifierChain(Vec<String>),
    String(String),
    Float(f64),
    Integer(i64),
    Bool(bool),
    Byte(u8),
    Prefix {
        op: Operator,
        exp: Box<Expression>,
    },
    Infix {
        op: Operator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        caller: Box<Expression>,
        args: Vec<Expression>,
    },
}

pub enum Statement {
    Assignment {
        identifier: String,
        expression: Expression,
    },
    Reassignment {
        identifier: String,
        expression: Expression,
    },
    Expression(Expression),
    Return(Expression),
}

pub enum StaticType {
    Str,
    Float,
    Int,
    Bool,
    Byte,
    Arr {
        inner: Box<StaticType>,
    },
    Map {
        keys: Box<StaticType>,
        values: Box<StaticType>,
    },
    Opt {
        inner: Box<StaticType>,
    },
    Res {
        inner: Box<StaticType>,
    },
}

struct Argument {
    identifier: String,
    r#type: StaticType,
}

pub struct FunctionSignature {
    pub name: String,
    pub args: Vec<Argument>,
    pub ret_type: StaticType,
}

pub struct Function {
    pub signature: FunctionSignature,
    pub statements: Vec<Statement>,
}

pub struct Import(pub Vec<String>);

pub struct Program {
    pub imports: Vec<Import>,
    pub functions: Vec<Function>,
}

pub type PrefixParseFn = fn() -> Expression;
pub type InfixParseFn = fn(Expression) -> Expression;

pub struct Parser<'a> {
    it: Peekable<Lexer<'a>>,
    prefix_fns: HashMap<Token, PrefixParseFn>,
    infix_fns: HashMap<Token, InfixParseFn>,
}

macro_rules! expect_token {
    ($parser:expr, $variant:path) => {{
        match $parser.read_token() {
            Some(Ok(tok)) => match tok {
                $variant(inner) => inner,
                other => return Err(parsingerr!("unexpected token: {:?}", other)),
            },
            Some(Err(e)) => return Err(e),
            None => return Err(parsingerr!("unexpected EOF")),
        }
    }};
}

impl<'a> Parser<'a> {
    pub fn new(l: Lexer<'a>) -> Self {
        let prefix_fns = HashMap::new();
        Self {
            it: l.peekable(),
            prefix_fns,
            infix_fns: HashMap::new(),
        }
    }

    pub fn from_source(src: &'a [u8]) -> Self {
        Self::new(Lexer::new(src))
    }

    fn read_token(&mut self) -> Option<Result<Token, ParsingError>> {
        self.it.next()
    }

    fn peek_token(&mut self) -> Option<&Result<Token, ParsingError>> {
        self.it.peek()
    }

    fn expect(&mut self, token: Token) -> Result<Token, ParsingError> {
        match self.read_token() {
            Some(Ok(t)) if t == token => Ok(t),
            Some(Ok(t)) => Err(parsingerr!("unexpected token: {:?}", t)),
            Some(Err(e)) => Err(e),
            None => Err(parsingerr!("unexpected EOF")),
        }
    }

    pub fn parse_expression(&mut self) -> Result<Expression, ParsingError> {
        todo!();
    }

    pub fn parse_identifier(&mut self) -> Result<Expression, ParsingError> {
        Ok(Expression::Identifier(expect_token!(self, Token::Ident)))
    }

    pub fn statement_tokens(&mut self) -> Result<Vec<Token>, ParsingError> {
        let mut tokens = Vec::new();
        loop {
            let t = self.read_token().ok_or(parsingerr!("unexpected EOF"))??;
            if t == Token::Semicolon {
                break;
            }
            tokens.push(t);
        }

        Ok(tokens)
    }

    pub fn parse_statement(&mut self) -> Result<Statement, ParsingError> {
        let expression = match self.peek_token().ok_or(parsingerr!("unexpected EOF"))? {
            Ok(Token::Return) => {
                self.read_token();
                Statement::Return(self.parse_expression()?)
            }
            Ok(Token::Ident(_)) => {
                let i = expect_token!(self, Token::Ident);
                match self.peek_token().ok_or(parsingerr!("unexpected EOF"))? {
                    Ok(Token::Assign) => {
                        self.read_token();
                        Statement::Assignment {
                            identifier: i,
                            expression: self.parse_expression()?,
                        }
                    }
                    Ok(Token::Equals) => {
                        self.read_token();
                        Statement::Reassignment {
                            identifier: i,
                            expression: self.parse_expression()?,
                        }
                    }
                    Ok(_) => Statement::Expression(self.parse_expression()?),
                    Err(err) => return Err(err.clone()),
                }
            }
            Ok(_) => Statement::Expression(self.parse_expression()?),
            Err(err) => return Err(err.clone()),
        };

        self.expect(Token::Semicolon)?;

        Ok(expression)
    }

    pub fn parse_return_statement(&mut self) -> Result<Statement, ParsingError> {
        todo!()
    }

    pub fn parse_imports(&mut self) -> Result<Vec<Import>, ParsingError> {
        let mut imports = Vec::new();

        while let Some(Ok(Token::Use)) = self.peek_token() {
            self.read_token();

            if let Some(Ok(Token::String(s))) = self.read_token() {
                let split = s.split("/").map(|ss| ss.to_string()).collect();
                imports.push(Import(split));
            } else {
                return Err(parsingerr!("expected string"));
            }

            self.expect(Token::Semicolon)?;
        }

        Ok(imports)
    }

    pub fn parse_type(&mut self) -> Result<StaticType, ParsingError> {
        fn parse_inner(p: &mut Parser) -> Result<StaticType, ParsingError> {
            p.expect(Token::LSqareBracket)?;
            let t = p.parse_type()?;
            p.expect(Token::RSqareBracket)?;
            Ok(t)
        }

        Ok(
            match self.read_token().ok_or(parsingerr!("unexpected EOF"))?? {
                Token::StrType => StaticType::Str,
                Token::FloatType => StaticType::Float,
                Token::IntType => StaticType::Int,
                Token::BoolType => StaticType::Bool,
                Token::ByteType => StaticType::Byte,
                Token::ArrayType => StaticType::Arr {
                    inner: Box::from(parse_inner(self)?),
                },
                Token::MapType => {
                    self.expect(Token::LSqareBracket)?;
                    let k = self.parse_type()?;
                    self.expect(Token::Comma)?;
                    let v = self.parse_type()?;
                    self.expect(Token::RSqareBracket)?;
                    StaticType::Map {
                        keys: Box::from(k),
                        values: Box::from(v),
                    }
                }
                Token::StructType => todo!(),
                Token::OptType => StaticType::Opt {
                    inner: Box::from(parse_inner(self)?),
                },
                Token::ResType => StaticType::Res {
                    inner: Box::from(parse_inner(self)?),
                },
                _ => Err(parsingerr!("expected type"))?,
            },
        )
    }

    pub fn parse_argument(&mut self) -> Result<Argument, ParsingError> {
        let name = expect_token!(self, Token::Ident);
        let tp = self.parse_type()?;

        Ok(Argument {
            identifier: name,
            r#type: tp,
        })
    }

    pub fn parse_function_signature(&mut self) -> Result<FunctionSignature, ParsingError> {
        self.expect(Token::FunctionDef)?;
        let name = expect_token!(self, Token::Ident);
        self.expect(Token::LParen)?;
        if let Some(Ok(Token::RParen)) = self.peek_token() {
            self.read_token();
            return Ok(FunctionSignature {
                name,
                args: Vec::new(),
                ret_type: self.parse_type()?,
            });
        }

        let mut args = Vec::new();
        loop {
            let arg_name = expect_token!(self, Token::Ident);
            let arg_type = self.parse_type()?;
            args.push(Argument {
                identifier: arg_name,
                r#type: arg_type,
            });
            match self.read_token().ok_or(parsingerr!("unexpected EOF"))?? {
                Token::Comma => {}
                Token::RParen => break,
                t => return Err(parsingerr!("unexpected token: {:?}", t)),
            }
        }

        Ok(FunctionSignature {
            name,
            args,
            ret_type: self.parse_type()?,
        })
    }

    pub fn parse_function(&mut self) -> Result<Function, ParsingError> {
        let signature = self.parse_function_signature()?;
        self.expect(Token::LBrace)?;

        if let Some(Ok(Token::RBrace)) = self.peek_token() {
            self.read_token();
            return Ok(Function {
                signature,
                statements: Vec::new(),
            });
        }

        let mut statements = Vec::new();
        loop {
            if let Some(Ok(Token::RBrace)) = self.peek_token() {
                self.read_token();
                return Ok(Function {
                    signature,
                    statements,
                });
            }

            statements.push(self.parse_statement()?);
        }
    }

    pub fn parse_functions(&mut self) -> Result<Vec<Function>, ParsingError> {
        let mut functions = Vec::new();

        while let Some(Ok(Token::FunctionDef)) = self.peek_token() {
            functions.push(self.parse_function()?);
        }

        Ok(functions)
    }

    pub fn parse_program(&mut self) -> Result<Program, ParsingError> {
        Ok(Program {
            imports: self.parse_imports()?,
            functions: self.parse_functions()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let src = b"f float := 3.14;";

        let mut p = Parser::from_source(src);

        println!("{:?}", p.statement_tokens().unwrap());
    }
}
