#[derive(Clone, Debug, PartialEq)]
pub struct ParsingError(pub String);

#[macro_export]
macro_rules! parsingerr {
    ($($arg:tt)*) => {
        ParsingError(format!($($arg)*))
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
