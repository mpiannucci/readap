use thiserror::Error;

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
    #[error("Not Implemented")]
    NotImplemented,
}
