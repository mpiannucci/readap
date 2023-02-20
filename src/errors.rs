use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid Data")]
	InvalidData,
    #[error("Parse Error")]
	ParseError,
    #[error("Invalid Typecast")]
	InvalidTypecast,
    #[error("Not Implemented")]
	NotImplemented,
}