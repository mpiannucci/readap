use std::fs;

use readap::{data::DataArray, parse_das_attributes, DdsDataset, DodsDataset};

#[test]
fn read_das() {
    let input = &fs::read_to_string("./data/swden/44097w9999.nc.das").unwrap();

    let attrs = parse_das_attributes(input).unwrap();

    assert_eq!(attrs.len(), 11);
    assert!(attrs.contains_key("time"));
    assert!(attrs.contains_key("wave_spectrum_r1"));
    assert!(attrs.contains_key("NC_GLOBAL"));

    let units: String = attrs["time"]["units"].clone().try_into().unwrap();
    assert_eq!(units, "seconds since 1970-01-01 00:00:00 UTC");
}

#[test]
fn read_dds() {
    let input = &fs::read_to_string("./data/44008.ncml.dds").unwrap();

    let dataset = DdsDataset::from_bytes(&input).unwrap();
    assert_eq!(dataset.values.len(), 16);
    assert_eq!(dataset.name, "data/stdmet/44008/44008.ncml");
}

#[test]
fn read_dataset() {
    let input = &fs::read("./data/swden/44097w9999.nc.dods").unwrap();

    let dataset = DodsDataset::from_bytes(&input).unwrap();

    // Once the dataset is parsed, data arrays can be extracted as DataArrays that encode the type in an enum.
    let mwd = if let DataArray::Int32(mwd) = dataset.variable_data("mean_wave_dir").unwrap() {
        mwd
    } else {
        vec![]
    };

    assert_eq!(mwd[0], 260);
    assert_eq!(mwd[mwd.len() - 1], 188);

    // The same goes for the coordinates for each array
    let coords = dataset.variable_coords("mean_wave_dir").unwrap();
    assert_eq!(coords[0].0, "time");
    assert_eq!(coords[1].0, "frequency");
    assert_eq!(coords[2].0, "latitude");
    assert_eq!(coords[3].0, "longitude");

    assert_eq!(
        TryInto::<Vec<i32>>::try_into(coords[0].1.clone()).unwrap()[2],
        1511910000
    );

    // Alternatively, the data arrays can be unpacked via an iterator. This is especially useful for zip interating multiple values at once
    let mwd_iter = dataset.variable_data_iter("mean_wave_dir").unwrap();
    let pwd_iter = dataset.variable_data_iter("principal_wave_dir").unwrap();

    let (mwd_fi, pwd_fi) = mwd_iter
        .zip(pwd_iter)
        .skip(5)
        .map(|(mwd_value, pwd_value)| {
            let mwd: i32 = mwd_value.try_into().unwrap();
            let pwd: i32 = pwd_value.try_into().unwrap();
            (mwd, pwd)
        })
        .next()
        .unwrap();

    assert_eq!(mwd_fi, 160);
    assert_eq!(pwd_fi, 168);

    let mwd_unpacked = dataset
        .variable_data_iter("mean_wave_dir")
        .unwrap()
        .map(|i| i.try_into().unwrap())
        .collect::<Vec<f64>>();

    assert_eq!(mwd_unpacked.len(), mwd.len());
}
