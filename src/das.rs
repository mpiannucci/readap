use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_till, take_until},
    character::complete::{multispace0, newline},
    multi::many_till,
    sequence::{preceded, terminated},
    IResult,
};

use crate::{
    data::{DataType, DataValue},
    errors::Error,
};

#[derive(Clone, Debug)]
pub struct DasAttribute {
    pub data_type: DataType,
    pub name: String,
    pub value: DataValue,
}

impl DasAttribute {
    pub fn parse(input: &str) -> IResult<&str, DasAttribute> {
        let (input, data_type) = DataType::parse(input)?;

        let (input, _) = multispace0(input)?;

        let (input, name) = take_till(char::is_whitespace)(input)?;
        let name = name.to_string();

        let (input, _) = multispace0(input)?;
        let (input, raw_value) = take_until(";")(input)?;
        let (input, _) = tag(";")(input)?;

        let value = match data_type {
            DataType::Byte => {
                let parsed = raw_value.parse::<i8>().map_err(|_| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
                })?;
                DataValue::Byte(parsed)
            }
            DataType::Int16 => {
                let parsed = raw_value.parse::<i16>().map_err(|_| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
                })?;
                DataValue::Int16(parsed)
            }
            DataType::UInt16 => {
                let parsed = raw_value.parse::<u16>().map_err(|_| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
                })?;
                DataValue::UInt16(parsed)
            }
            DataType::Int32 => {
                let parsed = raw_value.parse::<i32>().map_err(|_| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
                })?;
                DataValue::Int32(parsed)
            }
            DataType::UInt32 => {
                let parsed = raw_value.parse::<u32>().map_err(|_| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
                })?;
                DataValue::UInt32(parsed)
            }
            DataType::Float32 => {
                let parsed = raw_value.parse::<f32>().map_err(|_| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Float))
                })?;
                DataValue::Float32(parsed)
            }
            DataType::Float64 => {
                let parsed = raw_value.parse::<f64>().map_err(|_| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Float))
                })?;
                DataValue::Float64(parsed)
            }
            DataType::String => DataValue::String(raw_value.replace("\"", "")),
            DataType::URL => DataValue::URL(raw_value.replace("\"", "")),
        };

        Ok((
            input,
            DasAttribute {
                data_type,
                name,
                value,
            },
        ))
    }
}

impl TryInto<String> for DasAttribute {
    type Error = Error;

    fn try_into(self) -> Result<String, Self::Error> {
        self.value.try_into()
    }
}

impl TryInto<i32> for DasAttribute {
    type Error = Error;

    fn try_into(self) -> Result<i32, Self::Error> {
        self.value.try_into()
    }
}

impl TryInto<f32> for DasAttribute {
    type Error = Error;

    fn try_into(self) -> Result<f32, Self::Error> {
        self.value.try_into()
    }
}

pub type DasVariable = HashMap<String, DasAttribute>;

pub fn parse_das_variable(input: &str) -> IResult<&str, (String, DasVariable)> {
    let (input, name) = preceded(multispace0, take_till(char::is_whitespace))(input)?;
    let (input, _) = preceded(multispace0, tag("{"))(input)?;
    let (input, _) = newline(input)?;

    let (input, (attributes, _)) = many_till(
        preceded(multispace0, terminated(DasAttribute::parse, newline)),
        preceded(multispace0, tag("}")),
    )(input)?;

    let mut attrs = HashMap::new();
    attributes.into_iter().for_each(|a| {
        attrs.insert(a.name.clone(), a);
    });

    Ok((input, (name.to_string(), attrs)))
}

pub type DasAttributes = HashMap<String, DasVariable>;

fn parse_das_attributes_inner(input: &str) -> IResult<&str, DasAttributes> {
    let (input, _) = tag("Attributes {")(input)?;
    let (input, _) = newline(input)?;

    let (input, (vars, _)) = many_till(terminated(parse_das_variable, newline), tag("}"))(input)?;

    let mut attributes = HashMap::new();

    vars.into_iter().for_each(|(name, var)| {
        attributes.insert(name, var);
    });

    Ok((input, attributes))
}

pub fn parse_das_attributes(input: &str) -> Result<DasAttributes, Error> {
    match parse_das_attributes_inner(input) {
        Ok((_, a)) => Ok(a),
        Err(_) => Err(Error::ParseError),
    }
}

#[cfg(test)]
mod tests {
    use crate::{das::DataValue, data::DataType, errors::Error};

    use super::{parse_das_attributes, parse_das_variable, DasAttribute};

    #[test]
    fn parse_attribute() -> Result<(), Error> {
        let input = r#"String long_name "Longitude";"#;
        let (_, string_value) = DasAttribute::parse(input)?;
        assert_eq!(string_value.data_type, DataType::String);
        assert_eq!(string_value.name, "long_name");
        let value = if let DataValue::String(s) = string_value.value {
            s
        } else {
            "".to_string()
        };
        assert_eq!(value, "Longitude");

        let input = "Int32 _FillValue 999;";
        let (_, int_value) = DasAttribute::parse(input)?;
        assert_eq!(int_value.data_type, DataType::Int32);
        assert_eq!(int_value.name, "_FillValue");
        let value = if let DataValue::Int32(i) = int_value.value {
            i
        } else {
            0
        };
        assert_eq!(value, 999);

        let input = "Float32 _FillValue 999.0;";
        let (_, float_value) = DasAttribute::parse(input)?;
        assert_eq!(float_value.data_type, DataType::Float32);
        assert_eq!(float_value.name, "_FillValue");
        let value = if let DataValue::Float32(f) = float_value.value {
            f
        } else {
            0.0
        };
        assert!((value - 999.0).abs() < 0.0001);
        Ok(())
    }

    #[test]
    fn parse_variable() -> Result<(), Error> {
        let input = r#"    spectral_wave_density {
        String long_name "Spectral Wave Density";
        String short_name "swden";
        String standard_name "spectral_wave_density";
        String units "(meter * meter)/Hz";
        Float32 _FillValue 999.0;
    }"#;

        let (_, (name, attrs)) = parse_das_variable(input)?;
        assert_eq!(name, "spectral_wave_density");
        assert_eq!(attrs.len(), 5);
        assert_eq!(attrs["long_name"].data_type, DataType::String);
        assert_eq!(attrs["long_name"].name, "long_name");
        assert!(if let DataValue::String(s) = &attrs["long_name"].value {
            s == "Spectral Wave Density"
        } else {
            false
        });

        assert!(if let DataValue::Float32(f) = &attrs["_FillValue"].value {
            (f - 999.0).abs() < 0.0001
        } else {
            false
        });
        Ok(())
    }

    #[test]
    fn parse_das() -> Result<(), Error> {
        let input = r#"Attributes {
    time {
        String long_name "Epoch Time";
        String short_name "time";
        String standard_name "time";
        String units "seconds since 1970-01-01 00:00:00 UTC";
    }
    frequency {
        String long_name "Frequency";
        String short_name "frequency";
        String standard_name "frequency";
        String units "Hz";
    }
}"#;
        let attrs = parse_das_attributes(input)?;

        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains_key("time"));
        assert!(attrs.contains_key("frequency"));
        Ok(())
    }

    #[test]
    fn parse_invalid_attribute_values() {
        // Test invalid integer
        let input = "Int32 _FillValue not_a_number;";
        let result = DasAttribute::parse(input);
        assert!(result.is_err());

        // Test invalid float
        let input = "Float32 _FillValue invalid_float;";
        let result = DasAttribute::parse(input);
        assert!(result.is_err());

        // Test invalid byte
        let input = "Byte quality_flag 999;"; // Out of range for i8
        let result = DasAttribute::parse(input);
        assert!(result.is_err());
    }
}
