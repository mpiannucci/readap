use std::collections::HashMap;

use nom::{
    bytes::complete::{tag, take_till, take_until},
    character::complete::{multispace0, newline},
    multi::many_till,
    sequence::{preceded, terminated},
    IResult,
};

use crate::{
    data_type::{DataType, DataValue},
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
            DataType::Int32 => DataValue::Int32(raw_value.parse::<i32>().unwrap()),
            DataType::Float32 => DataValue::Float32(raw_value.parse::<f32>().unwrap()),
            DataType::String => DataValue::String(raw_value.replace("\"", "")),
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
    use crate::{das::DataValue, data_type::DataType};

    use super::{parse_das_attributes, parse_das_variable, DasAttribute};

    #[test]
    fn parse_attribute() {
        let input = r#"String long_name "Longitude";"#;
        let (_, string_value) = DasAttribute::parse(input).unwrap();
        assert_eq!(string_value.data_type, DataType::String);
        assert_eq!(string_value.name, "long_name");
        let value = if let DataValue::String(s) = string_value.value {
            s
        } else {
            "".to_string()
        };
        assert_eq!(value, "Longitude");

        let input = "Int32 _FillValue 999;";
        let (_, int_value) = DasAttribute::parse(input).unwrap();
        assert_eq!(int_value.data_type, DataType::Int32);
        assert_eq!(int_value.name, "_FillValue");
        let value = if let DataValue::Int32(i) = int_value.value {
            i
        } else {
            0
        };
        assert_eq!(value, 999);

        let input = "Float32 _FillValue 999.0;";
        let (_, float_value) = DasAttribute::parse(input).unwrap();
        assert_eq!(float_value.data_type, DataType::Float32);
        assert_eq!(float_value.name, "_FillValue");
        let value = if let DataValue::Float32(f) = float_value.value {
            f
        } else {
            0.0
        };
        assert!((value - 999.0).abs() < 0.0001);
    }

    #[test]
    fn parse_variable() {
        let input = r#"    spectral_wave_density {
        String long_name "Spectral Wave Density";
        String short_name "swden";
        String standard_name "spectral_wave_density";
        String units "(meter * meter)/Hz";
        Float32 _FillValue 999.0;
    }"#;

        let (_, (name, attrs)) = parse_das_variable(input).unwrap();
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
    }

    #[test]
    fn parse_das() {
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
        let attrs = parse_das_attributes(input).unwrap();

        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains_key("time"));
        assert!(attrs.contains_key("frequency"));
    }
}
