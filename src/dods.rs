use crate::{data_type::DataArray, dds::DdsDataset, errors::Error, DdsValue};

#[derive(Clone, Debug)]
pub struct DodsDataset {
    pub dds: DdsDataset,
    pub data_bytes: Vec<u8>,
}

impl DodsDataset {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let dods_string = String::from_utf8_lossy(&bytes);
        let (_, dds) = DdsDataset::parse(&dods_string).map_err(|_| Error::ParseError)?;

        let binary_data_start = match dods_string.find("Data:\n") {
            Some(p) => Ok(p),
            None => Err(Error::InvalidData),
        }? + 6;

        let data_bytes = bytes[binary_data_start..].to_vec();

        Ok(DodsDataset { dds, data_bytes })
    }

    pub fn variables(&self) -> Vec<String> {
        self.dds.values.iter().map(|v| v.name()).collect()
    }

    pub fn variable_index(&self, key: &str) -> Option<usize> {
        self.dds.values.iter().position(|v| v.name() == key)
    }

    pub fn variable_byte_offset(&self, key: &str) -> Option<usize> {
        let position = self.variable_index(key)?;
        let offset = (0usize..position).fold(0, |acc, i| acc + self.dds.values[i].byte_count());
        Some(offset)
    }

    pub fn variable_data(&self, key: &str) -> Result<DataArray, Error> {
        let index = match self.variable_index(key) {
            Some(o) => Ok(o),
            None => Err(Error::ParseError),
        }?;

        let offset = match self.variable_byte_offset(key) {
            Some(o) => Ok(o),
            None => Err(Error::ParseError),
        }?;

        let (_, data) = DataArray::parse(
            &self.data_bytes[offset..],
            &self.dds.values[index].array_data_type(),
        )
        .map_err(|_| Error::ParseError)?;

        Ok(data)
    }

    pub fn variable_coords(&self, key: &str) -> Option<Vec<String>> {
        let index = self.variable_index(key)?;
        Some(self.dds.values[index].coords())
    }

    pub fn variable_coord_data(&self, key: &str) -> Result<Vec<DataArray>, Error> {
        let position = match self.variable_byte_offset(key) {
            Some(p) => Ok(p),
            None => Err(Error::ParseError),
        }?;

        let index = match self.variable_index(key) {
            Some(i) => Ok(i),
            None => Err(Error::ParseError),
        }?;

        match &self.dds.values[index] {
            DdsValue::Array(a) => a.unpack_data(&self.data_bytes[position..]).map(|c| vec![c]),
            DdsValue::Grid(g) => g.unpack_coords_data(&self.data_bytes[position..]),
        }
    }
}
