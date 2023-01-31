use nom::{IResult, branch::alt, bytes::complete::tag};

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