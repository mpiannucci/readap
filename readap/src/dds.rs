use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{multispace0, newline},
    multi::many_till,
    sequence::{preceded, terminated},
    IResult,
};

use crate::{data::DataType, errors::Error};

#[derive(Clone, Debug)]
pub struct DdsArray {
    pub data_type: DataType,
    pub name: String,
    pub coords: Vec<(String, u32)>,
}

impl DdsArray {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, data_type) = DataType::parse(input)?;
        let (input, name) = take_until("[")(input)?;
        let name = name.trim().to_string();

        let (input, (coords, _)) = many_till(coordinate, tag(";"))(input)?;

        Ok((
            input,
            DdsArray {
                data_type,
                name,
                coords,
            },
        ))
    }

    pub fn array_length(&self) -> u32 {
        self.coords.iter().fold(1, |acc, c| acc * c.1)
    }

    pub fn byte_count(&self) -> usize {
        8 + self.array_length() as usize * self.data_type.byte_count()
    }
}

fn coordinate(input: &str) -> IResult<&str, (String, u32)> {
    let (input, _) = tag("[")(input)?;
    let (input, name) = take_until("=")(input)?;
    let name = name.trim();

    let (input, _) = tag("=")(input)?;
    let (input, len) = take_until("]")(input)?;
    let len = len.trim().parse::<u32>().unwrap();

    let (input, _) = tag("]")(input)?;

    Ok((input, (name.to_string(), len)))
}

#[derive(Clone, Debug)]
pub struct DdsGrid {
    pub name: String,
    pub array: DdsArray,
    pub coords: Vec<DdsArray>,
}

impl DdsGrid {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("Grid {")(input)?;
        let (input, _) = newline(input)?;
        let (input, _) = multispace0(input)?;

        let (input, _) = tag("ARRAY:")(input)?;
        let (input, _) = newline(input)?;
        let (input, _) = multispace0(input)?;

        let (input, array) = DdsArray::parse(input)?;
        let (input, _) = newline(input)?;
        let (input, _) = multispace0(input)?;

        let (input, _) = tag("MAPS:")(input)?;
        let (input, _) = newline(input)?;

        let (input, (coords, _)) = many_till(
            preceded(multispace0, terminated(DdsArray::parse, newline)),
            preceded(multispace0, tag("}")),
        )(input)?;

        let (input, name) = take_until(";")(input)?;
        let (input, _) = tag(";")(input)?;
        let name = name.trim().to_string();

        Ok((
            input,
            DdsGrid {
                name,
                array,
                coords,
            },
        ))
    }

    pub fn byte_count(&self) -> usize {
        let array_size = self.array.byte_count();
        self.coords
            .iter()
            .fold(array_size, |acc, c| acc + c.byte_count())
    }

    pub fn coords_offset(&self) -> usize {
        self.array.byte_count()
    }

    pub fn coord_offsets(&self) -> Vec<usize> {
        self.coords
            .iter()
            .scan(self.coords_offset(), |acc, c| {
                let prev = *acc;
                *acc += c.byte_count();
                Some(prev)
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct DdsStructure {
    pub name: String,
    pub fields: Vec<DdsValue>,
}

fn parse_simple_field(input: &str) -> IResult<&str, DdsValue> {
    preceded(
        multispace0,
        alt((
            // Try structure first
            |input| {
                let (input, structure) = DdsStructure::parse(input)?;
                let (input, _) = newline(input)?;
                Ok((input, DdsValue::Structure(structure)))
            },
            // Try sequence second
            |input| {
                let (input, sequence) = DdsSequence::parse(input)?;
                let (input, _) = newline(input)?;
                Ok((input, DdsValue::Sequence(sequence)))
            },
            // Try array with dimensions
            |input| {
                match DdsArray::parse(input) {
                    Ok((remaining, array)) => {
                        let (remaining, _) = newline(remaining)?;
                        Ok((remaining, DdsValue::Array(array)))
                    }
                    Err(_) => {
                        // Parse simple field like "Int32 id;" as a single-element array
                        let (input, data_type) = DataType::parse(input)?;
                        let (input, _) = multispace0(input)?;
                        let (input, name) = take_until(";")(input)?;
                        let (input, _) = tag(";")(input)?;
                        let (input, _) = newline(input)?;
                        let name = name.trim().to_string();

                        let array = DdsArray {
                            data_type,
                            name,
                            coords: vec![], // Simple scalar field
                        };
                        Ok((input, DdsValue::Array(array)))
                    }
                }
            },
        )),
    )(input)
}

impl DdsStructure {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("Structure {")(input)?;
        let (input, _) = newline(input)?;

        let (input, (fields, _)) =
            many_till(parse_simple_field, preceded(multispace0, tag("}")))(input)?;
        let (input, _) = multispace0(input)?;
        let (input, name) = take_until(";")(input)?;
        let (input, _) = tag(";")(input)?;
        let name = name.trim().to_string();

        Ok((input, DdsStructure { name, fields }))
    }

    pub fn byte_count(&self) -> usize {
        self.fields
            .iter()
            .fold(0, |acc, field| acc + field.byte_count())
    }
}

#[derive(Clone, Debug)]
pub struct DdsSequence {
    pub name: String,
    pub fields: Vec<DdsValue>,
}

impl DdsSequence {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("Sequence {")(input)?;
        let (input, _) = newline(input)?;

        let (input, (fields, _)) =
            many_till(parse_simple_field, preceded(multispace0, tag("}")))(input)?;
        let (input, _) = multispace0(input)?;
        let (input, name) = take_until(";")(input)?;
        let (input, _) = tag(";")(input)?;
        let name = name.trim().to_string();

        Ok((input, DdsSequence { name, fields }))
    }

    pub fn byte_count(&self) -> usize {
        // Sequences have variable length, so we return a base size
        // In practice, this would need to be calculated based on actual data
        8 + self
            .fields
            .iter()
            .fold(0, |acc, field| acc + field.byte_count())
    }
}

#[derive(Clone, Debug)]
pub enum DdsValue {
    Array(DdsArray),
    Grid(DdsGrid),
    Structure(DdsStructure),
    Sequence(DdsSequence),
}

impl DdsValue {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        preceded(
            multispace0,
            alt((
                Self::parse_structure,
                Self::parse_sequence,
                Self::parse_grid,
                Self::parse_array,
            )),
        )(input)
    }

    fn parse_array(input: &str) -> IResult<&str, DdsValue> {
        let (input, array) = terminated(DdsArray::parse, newline)(input)?;
        Ok((input, DdsValue::Array(array)))
    }

    fn parse_grid(input: &str) -> IResult<&str, DdsValue> {
        let (input, grid) = terminated(DdsGrid::parse, newline)(input)?;
        Ok((input, DdsValue::Grid(grid)))
    }

    fn parse_structure(input: &str) -> IResult<&str, DdsValue> {
        let (input, structure) = terminated(DdsStructure::parse, newline)(input)?;
        Ok((input, DdsValue::Structure(structure)))
    }

    fn parse_sequence(input: &str) -> IResult<&str, DdsValue> {
        let (input, sequence) = terminated(DdsSequence::parse, newline)(input)?;
        Ok((input, DdsValue::Sequence(sequence)))
    }

    pub fn name(&self) -> String {
        match self {
            Self::Array(a) => a.name.clone(),
            Self::Grid(g) => g.name.clone(),
            Self::Structure(s) => s.name.clone(),
            Self::Sequence(s) => s.name.clone(),
        }
    }

    pub fn byte_count(&self) -> usize {
        match self {
            DdsValue::Array(a) => a.byte_count(),
            DdsValue::Grid(g) => g.byte_count(),
            DdsValue::Structure(s) => s.byte_count(),
            DdsValue::Sequence(s) => s.byte_count(),
        }
    }

    pub fn array_data_type(&self) -> DataType {
        match self {
            DdsValue::Array(a) => a.data_type.clone(),
            DdsValue::Grid(g) => g.array.data_type.clone(),
            DdsValue::Structure(_) => panic!("Structure does not have a single data type"),
            DdsValue::Sequence(_) => panic!("Sequence does not have a single data type"),
        }
    }

    pub fn coords(&self) -> Vec<String> {
        match self {
            DdsValue::Array(a) => a.coords.iter().map(|c| c.0.clone()).collect(),
            DdsValue::Grid(g) => g.coords.iter().map(|c| c.name.clone()).collect(),
            DdsValue::Structure(_) => vec![], // Structures don't have coordinates
            DdsValue::Sequence(_) => vec![],  // Sequences don't have coordinates
        }
    }

    pub fn array(&self) -> Result<&DdsArray, Error> {
        match &self {
            DdsValue::Array(a) => Ok(a),
            _ => Err(Error::InvalidTypecast),
        }
    }

    pub fn grid(&self) -> Result<&DdsGrid, Error> {
        match &self {
            DdsValue::Grid(g) => Ok(g),
            _ => Err(Error::InvalidTypecast),
        }
    }

    pub fn structure(&self) -> Result<&DdsStructure, Error> {
        match &self {
            DdsValue::Structure(s) => Ok(s),
            _ => Err(Error::InvalidTypecast),
        }
    }

    pub fn sequence(&self) -> Result<&DdsSequence, Error> {
        match &self {
            DdsValue::Sequence(s) => Ok(s),
            _ => Err(Error::InvalidTypecast),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DdsDataset {
    pub name: String,
    pub values: Vec<DdsValue>,
}

impl DdsDataset {
    pub fn from_bytes(input: &str) -> Result<Self, Error> {
        match Self::parse(input) {
            Ok((_, d)) => Ok(d),
            Err(_) => Err(Error::ParseError),
        }
    }

    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, _) = tag("Dataset {")(input)?;
        let (input, _) = newline(input)?;

        let (input, (values, _)) = many_till(DdsValue::parse, tag("}"))(input)?;
        let (input, name) = take_until(";")(input)?;
        let (input, _) = tag(";")(input)?;
        let name = name.trim().to_string();

        Ok((input, DdsDataset { name, values }))
    }

    /// Create a new query builder
    pub fn query(&self, base_url: &str) -> crate::query::DatasetQuery<'_> {
        crate::query::DatasetQuery::new(self, base_url.to_string())
    }

    /// List all variable names in the dataset
    pub fn list_variables(&self) -> Vec<String> {
        self.values.iter().map(|v| v.name()).collect()
    }

    /// List all coordinate names in the dataset
    pub fn list_coordinates(&self) -> Vec<String> {
        let mut coords = std::collections::HashSet::new();

        for value in &self.values {
            match value {
                DdsValue::Array(array) => {
                    for (coord_name, _) in &array.coords {
                        coords.insert(coord_name.clone());
                    }
                }
                DdsValue::Grid(grid) => {
                    for coord in &grid.coords {
                        coords.insert(coord.name.clone());
                    }
                }
                _ => {} // Structures and sequences don't have coordinates
            }
        }

        coords.into_iter().collect()
    }

    /// Get detailed information about a variable
    pub fn get_variable_info(&self, name: &str) -> Option<crate::query::VariableInfo> {
        for value in &self.values {
            if value.name() == name {
                return Some(match value {
                    DdsValue::Array(array) => crate::query::VariableInfo {
                        name: array.name.clone(),
                        data_type: array.data_type.clone(),
                        coordinates: array.coords.iter().map(|(name, _)| name.clone()).collect(),
                        dimensions: array.coords.clone(),
                        variable_type: crate::query::VariableType::Array,
                    },
                    DdsValue::Grid(grid) => crate::query::VariableInfo {
                        name: grid.name.clone(),
                        data_type: grid.array.data_type.clone(),
                        coordinates: grid.coords.iter().map(|c| c.name.clone()).collect(),
                        dimensions: grid.array.coords.clone(),
                        variable_type: crate::query::VariableType::Grid,
                    },
                    DdsValue::Structure(structure) => crate::query::VariableInfo {
                        name: structure.name.clone(),
                        data_type: crate::data::DataType::String, // Structures don't have a single data type
                        coordinates: vec![],
                        dimensions: vec![],
                        variable_type: crate::query::VariableType::Structure,
                    },
                    DdsValue::Sequence(sequence) => crate::query::VariableInfo {
                        name: sequence.name.clone(),
                        data_type: crate::data::DataType::String, // Sequences don't have a single data type
                        coordinates: vec![],
                        dimensions: vec![],
                        variable_type: crate::query::VariableType::Sequence,
                    },
                });
            }
        }
        None
    }

    /// Get detailed information about a coordinate
    pub fn get_coordinate_info(&self, name: &str) -> Option<crate::query::CoordinateInfo> {
        let mut coord_info = None;
        let mut variables_using = Vec::new();

        // Find coordinate information and which variables use it
        for value in &self.values {
            match value {
                DdsValue::Array(array) => {
                    for (coord_name, coord_size) in &array.coords {
                        if coord_name == name {
                            if coord_info.is_none() {
                                coord_info = Some(crate::query::CoordinateInfo {
                                    name: coord_name.clone(),
                                    data_type: array.data_type.clone(), // Assume coordinate has same type as array
                                    size: *coord_size,
                                    variables_using: vec![],
                                });
                            }
                            variables_using.push(array.name.clone());
                        }
                    }
                }
                DdsValue::Grid(grid) => {
                    for coord in &grid.coords {
                        if coord.name == name {
                            if coord_info.is_none() {
                                coord_info = Some(crate::query::CoordinateInfo {
                                    name: coord.name.clone(),
                                    data_type: coord.data_type.clone(),
                                    size: coord.array_length(),
                                    variables_using: vec![],
                                });
                            }
                            variables_using.push(grid.name.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(mut info) = coord_info {
            info.variables_using = variables_using;
            Some(info)
        } else {
            None
        }
    }

    /// Check if a variable exists in the dataset
    pub fn has_variable(&self, name: &str) -> bool {
        self.values.iter().any(|v| v.name() == name)
    }

    /// Check if a coordinate exists in the dataset
    pub fn has_coordinate(&self, name: &str) -> bool {
        self.list_coordinates().contains(&name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::dds::{DataType, DdsValue};

    use super::{coordinate, DdsArray, DdsDataset, DdsGrid, DdsSequence, DdsStructure};

    #[test]
    fn parse_coords() {
        let coord = "[time = 7];";
        let (_, coords) = coordinate(coord).unwrap();
        assert_eq!(coords.0, "time");
        assert_eq!(coords.1, 7);
    }

    #[test]
    fn parse_array() {
        let single_array_input = "Int32 time[time = 7];";
        let (_, time_array) = DdsArray::parse(single_array_input).unwrap();

        assert_eq!(time_array.data_type, DataType::Int32);
        assert_eq!(time_array.name, "time");
        assert_eq!(time_array.coords[0].0, "time");
        assert_eq!(time_array.coords[0].1, 7);
        assert_eq!(time_array.array_length(), 7);

        let multi_array_input =
            "Float32 spectral_wave_density[time = 7][frequency = 64][latitude = 1][longitude = 1];";
        let (_, spectral_density_array) = DdsArray::parse(multi_array_input).unwrap();
        assert_eq!(spectral_density_array.data_type, DataType::Float32);
        assert_eq!(spectral_density_array.name, "spectral_wave_density");
        assert_eq!(spectral_density_array.coords.len(), 4);
        assert_eq!(spectral_density_array.coords[0].0, "time");
        assert_eq!(spectral_density_array.coords[0].1, 7);
        assert_eq!(spectral_density_array.coords[1].0, "frequency");
        assert_eq!(spectral_density_array.coords[1].1, 64);
        assert_eq!(spectral_density_array.coords[2].0, "latitude");
        assert_eq!(spectral_density_array.coords[2].1, 1);
        assert_eq!(spectral_density_array.coords[3].0, "longitude");
        assert_eq!(spectral_density_array.coords[3].1, 1);

        assert_eq!(spectral_density_array.array_length(), 7 * 64);
    }

    #[test]
    fn parse_grid() {
        let grid_input = r#"Grid {
     ARRAY:
        Float32 spectral_wave_density[time = 7][frequency = 64][latitude = 1][longitude = 1];
     MAPS:
        Int32 time[time = 7];
        Float32 frequency[frequency = 64];
        Float32 latitude[latitude = 1];
        Float32 longitude[longitude = 1];
    } spectral_wave_density;"#;

        let (_, grid) = DdsGrid::parse(grid_input).unwrap();

        assert_eq!(grid.name, "spectral_wave_density");
        assert_eq!(grid.array.data_type, DataType::Float32);
        assert_eq!(grid.array.name, "spectral_wave_density");
        assert_eq!(grid.array.coords.len(), 4);
        assert_eq!(grid.array.coords[0].0, "time");
        assert_eq!(grid.array.coords[0].1, 7);
        assert_eq!(grid.array.coords[1].0, "frequency");
        assert_eq!(grid.array.coords[1].1, 64);
        assert_eq!(grid.array.coords[2].0, "latitude");
        assert_eq!(grid.array.coords[2].1, 1);
        assert_eq!(grid.array.coords[3].0, "longitude");
        assert_eq!(grid.array.coords[3].1, 1);
        assert_eq!(grid.array.array_length(), 7 * 64);

        assert_eq!(grid.coords.len(), 4);
        assert_eq!(grid.coords[0].name, "time");
        assert_eq!(grid.coords[0].array_length(), 7);
        assert_eq!(grid.coords[1].name, "frequency");
        assert_eq!(grid.coords[1].array_length(), 64);
        assert_eq!(grid.coords[2].name, "latitude");
        assert_eq!(grid.coords[2].array_length(), 1);
        assert_eq!(grid.coords[3].name, "longitude");
        assert_eq!(grid.coords[3].array_length(), 1);
    }

    #[test]
    fn parse_dds() {
        let dataset_input = r#"Dataset {
    Int32 time[time = 7];
    Float32 frequency[frequency = 64];
    Grid {
     ARRAY:
        Float32 spectral_wave_density[time = 7][frequency = 64][latitude = 1][longitude = 1];
     MAPS:
        Int32 time[time = 7];
        Float32 frequency[frequency = 64];
        Float32 latitude[latitude = 1];
        Float32 longitude[longitude = 1];
    } spectral_wave_density;
} data/swden/44097/44097w9999.nc;
"#;

        let (_, dataset) = DdsDataset::parse(dataset_input).unwrap();

        assert_eq!(dataset.name, "data/swden/44097/44097w9999.nc");
        assert_eq!(dataset.values.len(), 3);
        assert!(if let DdsValue::Array(_) = dataset.values[0] {
            true
        } else {
            false
        });
        assert!(if let DdsValue::Array(_) = dataset.values[1] {
            true
        } else {
            false
        });
        assert!(if let DdsValue::Grid(_) = dataset.values[2] {
            true
        } else {
            false
        });
    }

    #[test]
    fn test_parse_new_data_type_arrays() {
        // Test Byte array
        let input = "Byte quality_flags[time = 10];";
        let (_, array) = DdsArray::parse(input).unwrap();
        assert_eq!(array.data_type, DataType::Byte);
        assert_eq!(array.name, "quality_flags");
        assert_eq!(array.coords.len(), 1);
        assert_eq!(array.coords[0].0, "time");
        assert_eq!(array.coords[0].1, 10);
        assert_eq!(array.array_length(), 10);
        assert_eq!(array.byte_count(), 8 + 10 * 1); // 8 bytes header + 10 bytes data

        // Test Int16 array
        let input = "Int16 elevations[latitude = 5][longitude = 5];";
        let (_, array) = DdsArray::parse(input).unwrap();
        assert_eq!(array.data_type, DataType::Int16);
        assert_eq!(array.name, "elevations");
        assert_eq!(array.coords.len(), 2);
        assert_eq!(array.array_length(), 25);
        assert_eq!(array.byte_count(), 8 + 25 * 2); // 8 bytes header + 50 bytes data

        // Test UInt32 array
        let input = "UInt32 file_sizes[files = 100];";
        let (_, array) = DdsArray::parse(input).unwrap();
        assert_eq!(array.data_type, DataType::UInt32);
        assert_eq!(array.name, "file_sizes");
        assert_eq!(array.coords.len(), 1);
        assert_eq!(array.array_length(), 100);
        assert_eq!(array.byte_count(), 8 + 100 * 4); // 8 bytes header + 400 bytes data

        // Test Float64 array
        let input = "Float64 precise_measurements[samples = 50];";
        let (_, array) = DdsArray::parse(input).unwrap();
        assert_eq!(array.data_type, DataType::Float64);
        assert_eq!(array.name, "precise_measurements");
        assert_eq!(array.coords.len(), 1);
        assert_eq!(array.array_length(), 50);
        assert_eq!(array.byte_count(), 8 + 50 * 8); // 8 bytes header + 400 bytes data
    }

    #[test]
    fn test_parse_structures_and_sequences() {
        // Test simple structure
        let input = r#"Structure {
    Int32 id;
    Float32 value;
} measurement;"#;
        let (_, structure) = DdsStructure::parse(input).unwrap();
        assert_eq!(structure.name, "measurement");
        assert_eq!(structure.fields.len(), 2);

        // Test simple sequence
        let input = r#"Sequence {
    Int32 timestamp;
    Float32 temperature;
} readings;"#;
        let (_, sequence) = DdsSequence::parse(input).unwrap();
        assert_eq!(sequence.name, "readings");
        assert_eq!(sequence.fields.len(), 2);
    }

    fn create_test_dataset() -> DdsDataset {
        let dds_content = r#"Dataset {
    Float32 latitude[latitude = 5];
    Float32 longitude[longitude = 10];
    Int32 time[time = 100];
    Grid {
     ARRAY:
        Float32 temperature[time = 100][latitude = 5][longitude = 10];
     MAPS:
        Int32 time[time = 100];
        Float32 latitude[latitude = 5];
        Float32 longitude[longitude = 10];
    } temperature;
    Grid {
     ARRAY:
        Float32 wind_speed[time = 100][latitude = 5][longitude = 10];
     MAPS:
        Int32 time[time = 100];
        Float32 latitude[latitude = 5];
        Float32 longitude[longitude = 10];
    } wind_speed;
} test_dataset;"#;

        DdsDataset::from_bytes(dds_content).unwrap()
    }

    #[test]
    fn test_dataset_metadata_methods() {
        let dataset = create_test_dataset();

        let variables = dataset.list_variables();
        assert_eq!(variables.len(), 5);
        assert!(variables.contains(&"latitude".to_string()));
        assert!(variables.contains(&"longitude".to_string()));
        assert!(variables.contains(&"time".to_string()));
        assert!(variables.contains(&"temperature".to_string()));
        assert!(variables.contains(&"wind_speed".to_string()));

        let coordinates = dataset.list_coordinates();
        assert_eq!(coordinates.len(), 3);
        assert!(coordinates.contains(&"latitude".to_string()));
        assert!(coordinates.contains(&"longitude".to_string()));
        assert!(coordinates.contains(&"time".to_string()));

        assert!(dataset.has_variable("temperature"));
        assert!(!dataset.has_variable("nonexistent"));

        assert!(dataset.has_coordinate("time"));
        assert!(!dataset.has_coordinate("nonexistent"));
    }

    #[test]
    fn test_variable_info() {
        let dataset = create_test_dataset();

        let temp_info = dataset.get_variable_info("temperature").unwrap();
        assert_eq!(temp_info.name, "temperature");
        assert_eq!(temp_info.data_type, crate::data::DataType::Float32);
        assert_eq!(temp_info.variable_type, crate::query::VariableType::Grid);
        assert_eq!(temp_info.coordinates, vec!["time", "latitude", "longitude"]);
        assert_eq!(temp_info.dimensions.len(), 3);
        assert_eq!(temp_info.dimensions[0], ("time".to_string(), 100));
        assert_eq!(temp_info.dimensions[1], ("latitude".to_string(), 5));
        assert_eq!(temp_info.dimensions[2], ("longitude".to_string(), 10));

        let lat_info = dataset.get_variable_info("latitude").unwrap();
        assert_eq!(lat_info.variable_type, crate::query::VariableType::Array);
        assert_eq!(lat_info.coordinates, vec!["latitude"]);
    }

    #[test]
    fn test_coordinate_info() {
        let dataset = create_test_dataset();

        let time_info = dataset.get_coordinate_info("time").unwrap();
        assert_eq!(time_info.name, "time");
        assert_eq!(time_info.size, 100);
        assert!(time_info
            .variables_using
            .contains(&"temperature".to_string()));
        assert!(time_info
            .variables_using
            .contains(&"wind_speed".to_string()));

        let lat_info = dataset.get_coordinate_info("latitude").unwrap();
        assert_eq!(lat_info.size, 5);
    }
}
