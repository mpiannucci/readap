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
}
