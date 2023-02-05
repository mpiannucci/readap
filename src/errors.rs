use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
	InvalidData,
	ParseError,
	InvalidTypecast,
	NotImplemented,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self {
            Error::InvalidData => "InvalidData",
            Error::ParseError => "ParseError",
            Error::InvalidTypecast => "InvalidTypecast",
            Error::NotImplemented => "NotImplemented",
        };

        f.write_str(message)
    }
}