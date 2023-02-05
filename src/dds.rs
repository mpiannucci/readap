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
                *acc = *acc + c.byte_count();
                Some(prev)
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
pub enum DdsValue {
    Array(DdsArray),
    Grid(DdsGrid),
}

impl DdsValue {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        preceded(multispace0, alt((Self::parse_array, Self::parse_grid)))(input)
    }

    fn parse_array(input: &str) -> IResult<&str, DdsValue> {
        let (input, array) = terminated(DdsArray::parse, newline)(input)?;
        Ok((input, DdsValue::Array(array)))
    }

    fn parse_grid(input: &str) -> IResult<&str, DdsValue> {
        let (input, grid) = terminated(DdsGrid::parse, newline)(input)?;
        Ok((input, DdsValue::Grid(grid)))
    }

    pub fn name(&self) -> String {
        match self {
            Self::Array(a) => a.name.clone(),
            Self::Grid(g) => g.name.clone(),
        }
    }

    pub fn byte_count(&self) -> usize {
        match self {
            DdsValue::Array(a) => a.byte_count(),
            DdsValue::Grid(g) => g.byte_count(),
        }
    }

    pub fn array_data_type(&self) -> DataType {
        match self {
            DdsValue::Array(a) => a.data_type.clone(),
            DdsValue::Grid(g) => g.array.data_type.clone(),
        }
    }

    pub fn coords(&self) -> Vec<String> {
        match self {
            DdsValue::Array(a) => a.coords.iter().map(|c| c.0.clone()).collect(),
            DdsValue::Grid(g) => g.coords.iter().map(|c| c.name.clone()).collect(),
        }
    }

    pub fn array(&self) -> Result<&DdsArray, Error> {
        match &self {
            DdsValue::Array(a) => Ok(a),
            DdsValue::Grid(_) => Err(Error::InvalidTypecast),
        }
    }

    pub fn grid(&self) -> Result<&DdsGrid, Error> {
        match &self {
            DdsValue::Array(_) => Err(Error::InvalidTypecast),
            DdsValue::Grid(g) => Ok(g),
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
}

#[cfg(test)]
mod tests {
    use crate::dds::{DataType, DdsValue};

    use super::{coordinate, DdsArray, DdsDataset, DdsGrid};

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
}
