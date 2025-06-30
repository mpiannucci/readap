use std::collections::HashMap;

use nom::{
    branch::alt,
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

#[derive(Clone, Debug)]
enum DasItem {
    Variable(String, DasVariable),
    GlobalAttribute(DasAttribute),
}

fn parse_das_item(input: &str) -> IResult<&str, DasItem> {
    alt((
        // Try to parse as a variable first (contains {})
        |input| {
            let (input, (name, var)) = parse_das_variable(input)?;
            Ok((input, DasItem::Variable(name, var)))
        },
        // If that fails, try to parse as a global attribute
        |input| {
            let (input, attr) = preceded(multispace0, DasAttribute::parse)(input)?;
            Ok((input, DasItem::GlobalAttribute(attr)))
        },
    ))(input)
}

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

    let (input, (items, _)) = many_till(terminated(parse_das_item, newline), tag("}"))(input)?;

    let mut attributes = HashMap::new();

    items.into_iter().for_each(|item| {
        match item {
            DasItem::Variable(name, var) => {
                attributes.insert(name, var);
            }
            DasItem::GlobalAttribute(attr) => {
                // Add global attributes to a special __global__ variable
                let global_key = "__global__".to_string();
                attributes
                    .entry(global_key)
                    .or_insert_with(HashMap::new)
                    .insert(attr.name.clone(), attr);
            }
        }
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

    #[test]
    fn test_parse_new_data_type_attributes() {
        // Test Byte attribute
        let input = "Byte quality_flag 1;";
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::Byte);
        assert_eq!(attr.name, "quality_flag");
        if let DataValue::Byte(val) = attr.value {
            assert_eq!(val, 1);
        } else {
            panic!("Expected Byte value");
        }

        // Test Int16 attribute
        let input = "Int16 elevation -500;";
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::Int16);
        assert_eq!(attr.name, "elevation");
        if let DataValue::Int16(val) = attr.value {
            assert_eq!(val, -500);
        } else {
            panic!("Expected Int16 value");
        }

        // Test UInt16 attribute
        let input = "UInt16 port_number 8080;";
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::UInt16);
        assert_eq!(attr.name, "port_number");
        if let DataValue::UInt16(val) = attr.value {
            assert_eq!(val, 8080);
        } else {
            panic!("Expected UInt16 value");
        }

        // Test UInt32 attribute
        let input = "UInt32 file_size 4294967295;";
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::UInt32);
        assert_eq!(attr.name, "file_size");
        if let DataValue::UInt32(val) = attr.value {
            assert_eq!(val, 4294967295);
        } else {
            panic!("Expected UInt32 value");
        }

        // Test Float64 attribute
        let input = "Float64 precision_value 3.141592653589793;";
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::Float64);
        assert_eq!(attr.name, "precision_value");
        if let DataValue::Float64(val) = attr.value {
            assert!((val - 3.141592653589793).abs() < 1e-15);
        } else {
            panic!("Expected Float64 value");
        }

        // Test URL attribute
        let input = r#"URL data_source "http://example.com/data.nc";"#;
        let (_, attr) = DasAttribute::parse(input).unwrap();
        assert_eq!(attr.data_type, DataType::URL);
        assert_eq!(attr.name, "data_source");
        if let DataValue::URL(val) = attr.value {
            assert_eq!(val, "http://example.com/data.nc");
        } else {
            panic!("Expected URL value");
        }
    }

    #[test]
    fn test_das_variable_with_new_types() -> Result<(), Error> {
        let input = r#"    sensor_data {
        Byte quality_flag 1;
        Int16 elevation -500;
        UInt16 port_number 8080;
        UInt32 file_size 4294967295;
        Float64 precision_value 3.141592653589793;
        URL data_source "http://example.com/data.nc";
    }"#;

        let (_, (name, attrs)) = parse_das_variable(input)?;
        assert_eq!(name, "sensor_data");
        assert_eq!(attrs.len(), 6);

        // Check each attribute type
        assert_eq!(attrs["quality_flag"].data_type, DataType::Byte);
        assert_eq!(attrs["elevation"].data_type, DataType::Int16);
        assert_eq!(attrs["port_number"].data_type, DataType::UInt16);
        assert_eq!(attrs["file_size"].data_type, DataType::UInt32);
        assert_eq!(attrs["precision_value"].data_type, DataType::Float64);
        assert_eq!(attrs["data_source"].data_type, DataType::URL);

        Ok(())
    }

    #[test]
    fn parse_real_gfs_das() -> Result<(), Error> {
        // Real DAS data from https://compute.earthmover.io/v1/services/dap2/earthmover-demos/gfs/main/solar/opendap.das
        let input = r#"Attributes {
    longitude {
        String axis "X";
        String standard_name "longitude";
        String units "degrees_east";
        Float64 _FillValue nan;
    }
    latitude {
        String axis "Y";
        String standard_name "latitude";
        String units "degrees_north";
        Float64 _FillValue nan;
    }
    time {
        String axis "T";
        String standard_name "forecast_reference_time";
        String units "hours since 2024-05-12T18:00:00";
        String calendar "proleptic_gregorian";
    }
    step {
        String standard_name "forecast_period";
        String units "hours";
    }
    t2m {
        Int32 GRIB_NV 0;
        Int32 GRIB_Nx 1440;
        Int32 GRIB_Ny 721;
        String GRIB_cfName "air_temperature";
        String GRIB_cfVarName "t2m";
        String GRIB_dataType "fc";
        String GRIB_gridDefinitionDescription "Latitude/longitude. Also called equidistant cylindrical, or Plate Carree";
        String GRIB_gridType "regular_ll";
        Float64 GRIB_iDirectionIncrementInDegrees 0.25;
        Int32 GRIB_iScansNegatively 0;
        Float64 GRIB_jDirectionIncrementInDegrees 0.25;
        Int32 GRIB_jPointsAreConsecutive 0;
        Int32 GRIB_jScansPositively 0;
        Float64 GRIB_latitudeOfFirstGridPointInDegrees 90.0;
        Float64 GRIB_latitudeOfLastGridPointInDegrees -90.0;
        Float64 GRIB_longitudeOfFirstGridPointInDegrees 0.0;
        Float64 GRIB_longitudeOfLastGridPointInDegrees 359.75;
        Float64 GRIB_missingValue 3.4028234663852886e+38;
        String GRIB_name "2 metre temperature";
        Int32 GRIB_numberOfPoints 1038240;
        Int32 GRIB_paramId 167;
        String GRIB_parameterName "Temperature";
        String GRIB_parameterUnits "K";
        String GRIB_shortName "2t";
        String GRIB_stepRange "0";
        String GRIB_stepType "instant";
        Int32 GRIB_stepUnits 1;
        String GRIB_typeOfLevel "heightAboveGround";
        String GRIB_units "K";
        String grid_mapping "gribfile_projection";
        String long_name "2 metre temperature";
        String standard_name "air_temperature";
        String units "K";
    }
    String description "GFS data ingested for forecasting demo";
}"#;

        let attrs = parse_das_attributes(input)?;

        // Verify basic structure
        assert!(attrs.contains_key("longitude"));
        assert!(attrs.contains_key("latitude"));
        assert!(attrs.contains_key("time"));
        assert!(attrs.contains_key("step"));
        assert!(attrs.contains_key("t2m"));

        // Check coordinate variables
        let longitude = &attrs["longitude"];
        assert!(longitude.contains_key("axis"));
        assert!(longitude.contains_key("standard_name"));
        assert!(longitude.contains_key("units"));

        // Check data variable with many attributes
        let t2m = &attrs["t2m"];
        assert!(t2m.contains_key("GRIB_name"));
        assert!(t2m.contains_key("units"));
        assert!(t2m.contains_key("standard_name"));

        // Check global description attribute
        // Note: This is a top-level attribute, not a variable

        Ok(())
    }
}
