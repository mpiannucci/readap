use nom::{
    branch::alt,
    bytes::complete::tag,
    multi::count,
    number::complete::{be_f32, be_f64, be_i16, be_i32, be_i8, be_u16, be_u32},
    IResult,
};

use crate::errors::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataType {
    Byte,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Float32,
    Float64,
    String,
    URL,
}

impl DataType {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, dtype) = alt((
            tag("Byte"),
            tag("Int16"),
            tag("UInt16"),
            tag("Int32"),
            tag("UInt32"),
            tag("Float32"),
            tag("Float64"),
            tag("String"),
            tag("URL"),
        ))(input)?;
        let dtype = match dtype {
            "Byte" => Self::Byte,
            "Int16" => Self::Int16,
            "UInt16" => Self::UInt16,
            "Int32" => Self::Int32,
            "UInt32" => Self::UInt32,
            "Float32" => Self::Float32,
            "Float64" => Self::Float64,
            "String" => Self::String,
            "URL" => Self::URL,
            _ => unreachable!(),
        };

        Ok((input, dtype))
    }

    pub fn byte_count(&self) -> usize {
        match self {
            DataType::Byte => 1,
            DataType::Int16 => 2,
            DataType::UInt16 => 2,
            DataType::Int32 => 4,
            DataType::UInt32 => 4,
            DataType::Float32 => 4,
            DataType::Float64 => 8,
            DataType::String => 0, // Variable length
            DataType::URL => 0,    // Variable length
        }
    }
}

#[derive(Clone, Debug)]
pub enum DataValue {
    Byte(i8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Float32(f32),
    Float64(f64),
    String(String),
    URL(String),
}

impl TryInto<String> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<String, Self::Error> {
        match &self {
            DataValue::String(s) => Ok(s.clone()),
            DataValue::URL(s) => Ok(s.clone()),
            _ => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<i32> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<i32, Self::Error> {
        match &self {
            DataValue::Byte(b) => Ok(*b as i32),
            DataValue::Int16(i) => Ok(*i as i32),
            DataValue::UInt16(u) => Ok(*u as i32),
            DataValue::Int32(i) => Ok(*i),
            DataValue::UInt32(u) => Ok(*u as i32),
            DataValue::Float32(f) => Ok(*f as i32),
            DataValue::Float64(f) => Ok(*f as i32),
            _ => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<i64> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<i64, Self::Error> {
        match &self {
            DataValue::Byte(b) => Ok(*b as i64),
            DataValue::Int16(i) => Ok(*i as i64),
            DataValue::UInt16(u) => Ok(*u as i64),
            DataValue::Int32(i) => Ok(*i as i64),
            DataValue::UInt32(u) => Ok(*u as i64),
            DataValue::Float32(f) => Ok(*f as i64),
            DataValue::Float64(f) => Ok(*f as i64),
            _ => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<f32> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<f32, Self::Error> {
        match &self {
            DataValue::Byte(b) => Ok(*b as f32),
            DataValue::Int16(i) => Ok(*i as f32),
            DataValue::UInt16(u) => Ok(*u as f32),
            DataValue::Int32(i) => Ok(*i as f32),
            DataValue::UInt32(u) => Ok(*u as f32),
            DataValue::Float32(f) => Ok(*f),
            DataValue::Float64(f) => Ok(*f as f32),
            _ => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<f64> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<f64, Self::Error> {
        match &self {
            DataValue::Byte(b) => Ok(*b as f64),
            DataValue::Int16(i) => Ok(*i as f64),
            DataValue::UInt16(u) => Ok(*u as f64),
            DataValue::Int32(i) => Ok(*i as f64),
            DataValue::UInt32(u) => Ok(*u as f64),
            DataValue::Float32(f) => Ok(*f as f64),
            DataValue::Float64(f) => Ok(*f),
            _ => Err(Error::InvalidTypecast),
        }
    }
}

pub struct DataValueIterator<'a> {
    input: &'a [u8],
    data_type: DataType,
    count: usize,
}

impl<'a> DataValueIterator<'a> {
    pub fn new(data: &'a [u8], data_type: DataType) -> Result<Self, Error> {
        // Check if the data type is supported for iteration
        match data_type {
            DataType::String | DataType::URL => {
                return Err(Error::NotImplemented);
            }
            _ => {}
        }

        let (input, count) =
            be_u32(data).map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)?;
        let (input, count_2) =
            be_u32(input).map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)?;

        assert!(count == count_2);

        Ok(Self {
            input,
            data_type,
            count: count as usize,
        })
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl<'a> Iterator for DataValueIterator<'a> {
    type Item = DataValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.len() < self.data_type.byte_count() {
            return None;
        }

        let (input, value) = match &self.data_type {
            DataType::Byte => be_i8(self.input)
                .map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)
                .map_or(None, |(input, b)| Some((input, DataValue::Byte(b)))),
            DataType::Int16 => be_i16(self.input)
                .map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)
                .map_or(None, |(input, i)| Some((input, DataValue::Int16(i)))),
            DataType::UInt16 => be_u16(self.input)
                .map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)
                .map_or(None, |(input, u)| Some((input, DataValue::UInt16(u)))),
            DataType::Int32 => be_i32(self.input)
                .map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)
                .map_or(None, |(input, i)| Some((input, DataValue::Int32(i)))),
            DataType::UInt32 => be_u32(self.input)
                .map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)
                .map_or(None, |(input, u)| Some((input, DataValue::UInt32(u)))),
            DataType::Float32 => be_f32(self.input)
                .map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)
                .map_or(None, |(input, f)| Some((input, DataValue::Float32(f)))),
            DataType::Float64 => be_f64(self.input)
                .map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)
                .map_or(None, |(input, f)| Some((input, DataValue::Float64(f)))),
            DataType::String | DataType::URL => {
                // These types are not supported for iteration and should be caught in new()
                unreachable!("String and URL types should be rejected in DataValueIterator::new()")
            }
        }?;

        self.input = input;
        Some(value)
    }
}

#[derive(Clone, Debug)]
pub enum DataArray {
    Byte(Vec<i8>),
    Int16(Vec<i16>),
    UInt16(Vec<u16>),
    Int32(Vec<i32>),
    UInt32(Vec<u32>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    String(Vec<String>),
    URL(Vec<String>),
}

impl DataArray {
    pub fn parse(input: &[u8], data_type: DataType) -> IResult<&[u8], Self> {
        let (input, length) = be_u32(input)?;
        let (input, length_2) = be_u32(input)?;

        assert!(length == length_2);

        match data_type {
            DataType::Byte => {
                let (input, values) = count(be_i8, length as usize)(input)?;
                Ok((input, Self::Byte(values)))
            }
            DataType::Int16 => {
                let (input, values) = count(be_i16, length as usize)(input)?;
                Ok((input, Self::Int16(values)))
            }
            DataType::UInt16 => {
                let (input, values) = count(be_u16, length as usize)(input)?;
                Ok((input, Self::UInt16(values)))
            }
            DataType::Int32 => {
                let (input, values) = count(be_i32, length as usize)(input)?;
                Ok((input, Self::Int32(values)))
            }
            DataType::UInt32 => {
                let (input, values) = count(be_u32, length as usize)(input)?;
                Ok((input, Self::UInt32(values)))
            }
            DataType::Float32 => {
                let (input, values) = count(be_f32, length as usize)(input)?;
                Ok((input, Self::Float32(values)))
            }
            DataType::Float64 => {
                let (input, values) = count(be_f64, length as usize)(input)?;
                Ok((input, Self::Float64(values)))
            }
            DataType::String => {
                // String array parsing is not implemented
                Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )))
            }
            DataType::URL => {
                // URL array parsing is not implemented
                Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )))
            }
        }
    }
}

impl TryInto<Vec<i32>> for DataArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i32>, Self::Error> {
        match self {
            DataArray::Byte(v) => Ok(v.into_iter().map(|i| i as i32).collect()),
            DataArray::Int16(v) => Ok(v.into_iter().map(|i| i as i32).collect()),
            DataArray::UInt16(v) => Ok(v.into_iter().map(|i| i as i32).collect()),
            DataArray::Int32(v) => Ok(v),
            DataArray::UInt32(v) => Ok(v.into_iter().map(|i| i as i32).collect()),
            DataArray::Float32(v) => Ok(v.into_iter().map(|i| i as i32).collect()),
            DataArray::Float64(v) => Ok(v.into_iter().map(|i| i as i32).collect()),
            _ => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<Vec<i64>> for DataArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i64>, Self::Error> {
        match self {
            DataArray::Byte(v) => Ok(v.into_iter().map(|i| i as i64).collect()),
            DataArray::Int16(v) => Ok(v.into_iter().map(|i| i as i64).collect()),
            DataArray::UInt16(v) => Ok(v.into_iter().map(|i| i as i64).collect()),
            DataArray::Int32(v) => Ok(v.into_iter().map(|i| i as i64).collect()),
            DataArray::UInt32(v) => Ok(v.into_iter().map(|i| i as i64).collect()),
            DataArray::Float32(v) => Ok(v.into_iter().map(|i| i as i64).collect()),
            DataArray::Float64(v) => Ok(v.into_iter().map(|i| i as i64).collect()),
            _ => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<Vec<f32>> for DataArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<f32>, Self::Error> {
        match self {
            DataArray::Byte(v) => Ok(v.into_iter().map(|i| i as f32).collect()),
            DataArray::Int16(v) => Ok(v.into_iter().map(|i| i as f32).collect()),
            DataArray::UInt16(v) => Ok(v.into_iter().map(|i| i as f32).collect()),
            DataArray::Int32(v) => Ok(v.into_iter().map(|i| i as f32).collect()),
            DataArray::UInt32(v) => Ok(v.into_iter().map(|i| i as f32).collect()),
            DataArray::Float32(v) => Ok(v),
            DataArray::Float64(v) => Ok(v.into_iter().map(|i| i as f32).collect()),
            _ => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<Vec<f64>> for DataArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<f64>, Self::Error> {
        match self {
            DataArray::Byte(v) => Ok(v.into_iter().map(|i| i as f64).collect()),
            DataArray::Int16(v) => Ok(v.into_iter().map(|i| i as f64).collect()),
            DataArray::UInt16(v) => Ok(v.into_iter().map(|i| i as f64).collect()),
            DataArray::Int32(v) => Ok(v.into_iter().map(|i| i as f64).collect()),
            DataArray::UInt32(v) => Ok(v.into_iter().map(|i| i as f64).collect()),
            DataArray::Float32(v) => Ok(v.into_iter().map(|i| i as f64).collect()),
            DataArray::Float64(v) => Ok(v),
            _ => Err(Error::InvalidTypecast),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_data_type() {
        let (_, data_type) = DataType::parse("Int32").unwrap();
        assert_eq!(data_type, DataType::Int32);

        let (_, data_type) = DataType::parse("Float32").unwrap();
        assert_eq!(data_type, DataType::Float32);
    }

    #[test]
    fn test_not_implemented_data_value_iterator() {
        // Test that String and URL types return NotImplemented error
        let dummy_data = [0u8; 16]; // Some dummy data

        let result = DataValueIterator::new(&dummy_data, DataType::String);
        assert!(matches!(result, Err(Error::NotImplemented)));

        let result = DataValueIterator::new(&dummy_data, DataType::URL);
        assert!(matches!(result, Err(Error::NotImplemented)));

        // Test that supported types work
        let result = DataValueIterator::new(&dummy_data, DataType::Int32);
        assert!(result.is_ok());
    }
}
