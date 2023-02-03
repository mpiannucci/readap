use nom::{
    branch::alt,
    bytes::complete::tag,
    multi::count,
    number::complete::{be_f32, be_i32, be_u32},
    IResult,
};

use crate::errors::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DataType {
    Int32,
    Float32,
    String,
}

impl DataType {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, dtype) = alt((tag("Int32"), tag("Float32"), tag("String")))(input)?;
        let dtype = match dtype {
            "Int32" => Self::Int32,
            "Float32" => Self::Float32,
            "String" => Self::String,
            _ => unreachable!(),
        };

        Ok((input, dtype))
    }

    pub fn byte_count(&self) -> usize {
        match self {
            DataType::Int32 => 4,
            DataType::Float32 => 4,
            DataType::String => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DataValue {
    Int32(i32),
    Float32(f32),
    String(String),
}

impl TryInto<String> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<String, Self::Error> {
        if let DataValue::String(s) = &self {
            Ok(s.clone())
        } else {
            Err(Error::InvalidTypecast)
        }
    }
}

impl TryInto<i32> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<i32, Self::Error> {
        if let DataValue::Int32(i) = &self {
            Ok(*i)
        } else {
            Err(Error::InvalidTypecast)
        }
    }
}

impl TryInto<f32> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<f32, Self::Error> {
        if let DataValue::Float32(f) = &self {
            Ok(*f)
        } else {
            Err(Error::InvalidTypecast)
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
}

impl<'a> Iterator for DataValueIterator<'a> {
    type Item = DataValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.len() < self.data_type.byte_count() {
            return None;
        }

        let (input, value) = match &self.data_type {
            DataType::Int32 => {
                be_i32(self.input)
                    .map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)
                    .map_or(None, |(input, i)| Some((input, DataValue::Int32(i))))
            }
            DataType::Float32 => {
                be_f32(self.input)
                    .map_err(|_: nom::Err<nom::error::Error<_>>| Error::ParseError)
                    .map_or(None, |(input, f)| Some((input, DataValue::Float32(f))))
            }
            DataType::String => unreachable!(),
        }?;

        self.input = input;
        Some(value)
    }
}

#[derive(Clone, Debug)]
pub enum DataArray {
    Int32(Vec<i32>),
    Float32(Vec<f32>),
}

impl DataArray {
    pub fn parse<'a>(input: &'a [u8], data_type: DataType) -> IResult<&'a [u8], Self> {
        let (input, length) = be_u32(input)?;
        let (input, length_2) = be_u32(input)?;

        assert!(length == length_2);

        match data_type {
            DataType::Int32 => {
                let (input, values) = count(be_i32, length as usize)(input)?;
                Ok((input, Self::Int32(values)))
            }
            DataType::Float32 => {
                let (input, values) = count(be_f32, length as usize)(input)?;
                Ok((input, Self::Float32(values)))
            }
            DataType::String => unreachable!(),
        }
    }
}

impl TryInto<Vec<i32>> for DataArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i32>, Self::Error> {
        match self {
            DataArray::Int32(v) => Ok(v),
            DataArray::Float32(_) => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<Vec<f32>> for DataArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<f32>, Self::Error> {
        match self {
            DataArray::Int32(_) => Err(Error::InvalidTypecast),
            DataArray::Float32(v) => Ok(v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DataType;

    #[test]
    fn parse_data_type() {
        let input = "Int32";
        let (_, dtype) = DataType::parse(input).unwrap();
        assert_eq!(dtype, DataType::Int32);

        let input = "Float32";
        let (_, dtype) = DataType::parse(input).unwrap();
        assert_eq!(dtype, DataType::Float32);

        let input = "String";
        let (_, dtype) = DataType::parse(input).unwrap();
        assert_eq!(dtype, DataType::String);
    }
}
