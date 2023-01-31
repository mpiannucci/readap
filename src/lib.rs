pub mod data_type;
pub mod das;
pub mod dds;

#[cfg(test)]
mod tests {
    use byteorder::{BigEndian, ReadBytesExt};
    use std::{fs, io::Cursor};

    use crate::dds::DdsDataset;

    #[test]
    fn read_dods_map() {
        let data = &fs::read("./data/simple_grid/44097w9999.nc.dods").unwrap();
        let dods_string = String::from_utf8_lossy(data);
        let position = dods_string.find("Data:\n").unwrap();

        let (_, dataset) = DdsDataset::parse(&dods_string).unwrap();
        assert_eq!(dataset.name, "data/swden/44097/44097w9999.nc");
        assert_eq!(dataset.values.len(), 1);

        let mut reader = Cursor::new(&data[position + 6..]);

        let len = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(len, 448);

        let len_second = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(len_second, 448);

        let data = (0..len)
            .map(|_| reader.read_i32::<BigEndian>().unwrap())
            .collect::<Vec<i32>>();

        assert_eq!(data[0], 260);
        assert_eq!(data[data.len() - 1], 188);

        let time_len = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(time_len, 7);

        let time_len_second = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(time_len_second, 7);

        let time = (0..time_len)
            .map(|_| reader.read_i32::<BigEndian>().unwrap())
            .collect::<Vec<i32>>();

        let time_truth = vec![
            1511902800, 1511906400, 1511910000, 1511913600, 1511917200, 1511920800, 1511924400,
        ];
        for i in 0..time.len() {
            assert_eq!(time[i], time_truth[i]);
        }

        let freq_len = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(freq_len, 64);

        let freq_len_second = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(freq_len_second, 64);

        let frequencies = (0..freq_len)
            .map(|_| reader.read_f32::<BigEndian>().unwrap())
            .collect::<Vec<f32>>();

        assert!((frequencies[0] - 0.025).abs() < 0.00001);
        assert!((frequencies[frequencies.len() - 1] - 0.58).abs() < 0.00001);

        let lat_len = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(lat_len, 1);

        let lat_len_second = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(lat_len_second, 1);

        let lats = (0..lat_len)
            .map(|_| reader.read_f32::<BigEndian>().unwrap())
            .collect::<Vec<f32>>();

        assert!((lats[0] - 40.969).abs() < 0.00001);

        let lng_len = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(lng_len, 1);

        let lng_len_second = reader.read_u32::<BigEndian>().unwrap();
        assert_eq!(lng_len_second, 1);

        let lngs = (0..lng_len)
            .map(|_| reader.read_f32::<BigEndian>().unwrap())
            .collect::<Vec<f32>>();

        assert!((lngs[0] - -71.127).abs() < 0.00001);
    }
}
