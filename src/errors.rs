use thiserror::Error;
use nom;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid Data")]
    InvalidData,
    #[error("Parse Error")]
    ParseError,
    #[error("Invalid Typecast")]
    InvalidTypecast,
    #[error("Invalid Attribute Value: {0}")]
    InvalidAttributeValue(String),
    #[error("Nom Parse Error: {0}")]
    NomError(String),
    #[error("Not Implemented")]
    NotImplemented,
}

// Convert nom errors to our custom Error type
impl<I> From<nom::Err<nom::error::Error<I>>> for Error 
where
    I: std::fmt::Debug,
{
    fn from(err: nom::Err<nom::error::Error<I>>) -> Self {
        Error::NomError(format!("{err:?}"))
    }
}

// Convert nom::error::Error to our custom Error type
impl<I> From<nom::error::Error<I>> for Error 
where
    I: std::fmt::Debug,
{
    fn from(err: nom::error::Error<I>) -> Self {
        Error::NomError(format!("{err:?}"))
    }
}
