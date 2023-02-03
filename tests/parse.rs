use std::fs;

use readap::{data_type::DataArray, parse_das_attributes, DdsDataset, DodsDataset};

#[test]
fn read_das() {
    let input = &fs::read_to_string("./data/swden/44097w9999.nc.das").unwrap();

    let attrs = parse_das_attributes(input).unwrap();

    assert_eq!(attrs.len(), 11);
    assert!(attrs.contains_key("time"));
    assert!(attrs.contains_key("wave_spectrum_r1"));
    assert!(attrs.contains_key("NC_GLOBAL"));
}

#[test]
fn read_dds() {
    let input = &fs::read_to_string("./data/44008.ncml.dds").unwrap();

    let (_, dataset) = DdsDataset::parse(input).unwrap();
    assert_eq!(dataset.values.len(), 16);
    assert_eq!(dataset.name, "data/stdmet/44008/44008.ncml");
}

#[test]
fn read_dataset() {
    let input = &fs::read("./data/swden/44097w9999.nc.dods").unwrap();

    let dataset = DodsDataset::from_bytes(&input).unwrap();
    let mwd = if let DataArray::Int32(mwd) = dataset.variable_data("mean_wave_dir").unwrap() {
        mwd
    } else {
        vec![]
    };

    assert_eq!(mwd[0], 260);
    assert_eq!(mwd[mwd.len() - 1], 188);

    let coords = dataset.variable_coords("mean_wave_dir").unwrap();
    assert_eq!(coords[0].0, "time");
    assert_eq!(coords[1].0, "frequency");
    assert_eq!(coords[2].0, "latitude");
    assert_eq!(coords[3].0, "longitude");

    assert_eq!( TryInto::<Vec<i32>>::try_into(coords[0].1.clone()).unwrap()[2], 1511910000);
}
