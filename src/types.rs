#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Illegal,
    Ident(String),
    Assign,
    // punctuation
    Semicolon,
    Colon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    // keywords
    FunctionDef,
    // builtin type identifiers
    StrType,
    FloatType,
    IntType,
    BoolType,
    // type val
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

// TODO: refactor using vvv
#[derive(Clone, Debug, PartialEq)]
pub struct ParsingError(pub String);

#[macro_export]
macro_rules! parsingerr {
    ($($arg:tt)*) => {
        PdfError(format!($($arg)*))
    };
}

impl<E> From<E> for ParsingError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(e: E) -> Self {
        Self(e.to_string())
    }
}
