use crate::{
    data::{DataArray, DataValueIterator},
    dds::DdsDataset,
    errors::Error,
    DdsValue,
};

#[derive(Clone, Debug)]
pub struct DodsDataset<'a> {
    pub dds: DdsDataset,
    pub data_bytes: &'a [u8],
}

impl<'a> DodsDataset<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, Error> {
        let dods_string = String::from_utf8_lossy(bytes);
        let (_, dds) = DdsDataset::parse(&dods_string).map_err(|_| Error::ParseError)?;

        let binary_data_start = match dods_string.find("Data:\n") {
            Some(p) => Ok(p),
            None => Err(Error::InvalidData),
        }? + 6;

        let data_bytes = &bytes[binary_data_start..];

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

    pub fn variable_data_iter(&self, key: &str) -> Result<DataValueIterator, Error> {
        let index = match self.variable_index(key) {
            Some(o) => Ok(o),
            None => Err(Error::ParseError),
        }?;

        let offset = match self.variable_byte_offset(key) {
            Some(o) => Ok(o),
            None => Err(Error::ParseError),
        }?;

        match &self.dds.values[index] {
            DdsValue::Array(a) => DataValueIterator::new(
                &self.data_bytes[offset..offset + a.byte_count()],
                a.data_type.clone(),
            ),
            DdsValue::Grid(g) => DataValueIterator::new(
                &self.data_bytes[offset..offset + g.array.byte_count()],
                g.array.data_type.clone(),
            ),
            DdsValue::Structure(_) => Err(Error::NotImplemented),
            DdsValue::Sequence(_) => Err(Error::NotImplemented),
        }
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

        let (_, data) = match &self.dds.values[index] {
            DdsValue::Array(a) => DataArray::parse(&self.data_bytes[offset..], a.data_type.clone()),
            DdsValue::Grid(g) => {
                DataArray::parse(&self.data_bytes[offset..], g.array.data_type.clone())
            }
            DdsValue::Structure(_) => return Err(Error::NotImplemented),
            DdsValue::Sequence(_) => return Err(Error::NotImplemented),
        }
        .map_err(|_| Error::ParseError)?;

        Ok(data)
    }

    pub fn variable_coords(&self, key: &str) -> Result<Vec<(String, DataArray)>, Error> {
        let position = match self.variable_byte_offset(key) {
            Some(p) => Ok(p),
            None => Err(Error::ParseError),
        }?;

        let index = match self.variable_index(key) {
            Some(i) => Ok(i),
            None => Err(Error::ParseError),
        }?;

        match &self.dds.values[index] {
            DdsValue::Array(a) => {
                let name = a.name.clone();
                DataArray::parse(&self.data_bytes[position..], a.data_type.clone())
                    .map_err(|_| Error::ParseError)
                    .map(|(_, a)| vec![(name, a)])
            }
            DdsValue::Grid(g) => g
                .coords
                .iter()
                .scan(g.coords_offset(), |acc, c| {
                    let name = c.name.clone();
                    let data =
                        DataArray::parse(&self.data_bytes[position + *acc..], c.data_type.clone())
                            .map_err(|_| Error::ParseError)
                            .map(|(_, a)| (name, a));
                    *acc += c.byte_count();
                    Some(data)
                })
                .collect(),
            DdsValue::Structure(_) => Err(Error::NotImplemented),
            DdsValue::Sequence(_) => Err(Error::NotImplemented),
        }
    }
}
