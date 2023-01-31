use std::fs;

use readap::{das::parse_das_attributes, dds::DdsDataset};

#[test]
fn read_das() {
    let input = &fs::read_to_string("./data/swden/44097w9999.nc.das").unwrap();

    let (_, attrs) = parse_das_attributes(input).unwrap();

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