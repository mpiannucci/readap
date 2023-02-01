use nom::{branch::alt, bytes::complete::tag, IResult, number::complete::{be_u32, be_i32, be_f32}, multi::{count}};

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
pub enum DataArray {
	Int32(Vec<i32>), 
	Float32(Vec<f32>),
}

impl DataArray {
	pub fn parse<'a>(input: &'a [u8], data_type: &'a DataType) -> IResult<&'a [u8], Self> {
		let (input, length) = be_u32(input)?;
		let (input, length_2) = be_u32(input)?;

		assert!(length == length_2);

		match data_type {
    		DataType::Int32 => {
    			let (input, values) = count(be_i32, length as usize)(input)?;
    			Ok((input, Self::Int32(values)))
    		},
    		DataType::Float32 => {
    			let (input, values) = count(be_f32, length as usize)(input)?;
    			Ok((input, Self::Float32(values)))
    		},
    		DataType::String => unreachable!(),
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataArrayError {
	ParseError,
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
