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
        match &self {
            DataValue::Int32(i) => Ok(*i),
            DataValue::Float32(f) => Ok(*f as i32),
            DataValue::String(_) => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<i64> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<i64, Self::Error> {
        match &self {
            DataValue::Int32(i) => Ok(*i as i64),
            DataValue::Float32(f) => Ok(*f as i64),
            DataValue::String(_) => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<f32> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<f32, Self::Error> {
        match &self {
            DataValue::Int32(i) => Ok(*i as f32),
            DataValue::Float32(f) => Ok(*f),
            DataValue::String(_) => Err(Error::InvalidTypecast),
        }
    }
}

impl TryInto<f64> for DataValue {
    type Error = Error;

    fn try_into(self) -> Result<f64, Self::Error> {
        match &self {
            DataValue::Int32(i) => Ok(*i as f64),
            DataValue::Float32(f) => Ok(*f as f64),
            DataValue::String(_) => Err(Error::InvalidTypecast),
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
            DataArray::Float32(v) => Ok(v.into_iter().map(|i| i as i32).collect()),
        }
    }
}

impl TryInto<Vec<i64>> for DataArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<i64>, Self::Error> {
        match self {
            DataArray::Int32(v) => Ok(v.into_iter().map(|i| i as i64).collect()),
            DataArray::Float32(v) => Ok(v.into_iter().map(|i| i as i64).collect()),
        }
    }
}

impl TryInto<Vec<f32>> for DataArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<f32>, Self::Error> {
        match self {
            DataArray::Int32(v) => Ok(v.into_iter().map(|i| i as f32).collect()),
            DataArray::Float32(v) => Ok(v),
        }
    }
}

impl TryInto<Vec<f64>> for DataArray {
    type Error = Error;

    fn try_into(self) -> Result<Vec<f64>, Self::Error> {
        match self {
            DataArray::Int32(v) => Ok(v.into_iter().map(|i| i as f64).collect()),
            DataArray::Float32(v) => Ok(v.into_iter().map(|i| i as f64).collect()),
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
