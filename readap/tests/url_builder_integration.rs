use readap::{Constraint, IndexRange, UrlBuilder};

#[test]
fn test_real_world_opendap_urls() {
    let base_url = "https://thredds.ucar.edu/thredds/dodsC/grib/NCEP/GFS/Global_0p25deg/GFS_Global_0p25deg_20231201_0000.grib2";

    let builder = UrlBuilder::new(base_url);

    assert_eq!(
        builder.das_url(),
        "https://thredds.ucar.edu/thredds/dodsC/grib/NCEP/GFS/Global_0p25deg/GFS_Global_0p25deg_20231201_0000.grib2.das"
    );

    assert_eq!(
        builder.dds_url(),
        "https://thredds.ucar.edu/thredds/dodsC/grib/NCEP/GFS/Global_0p25deg/GFS_Global_0p25deg_20231201_0000.grib2.dds"
    );

    assert_eq!(
        builder.dods_url().unwrap(),
        "https://thredds.ucar.edu/thredds/dodsC/grib/NCEP/GFS/Global_0p25deg/GFS_Global_0p25deg_20231201_0000.grib2.dods"
    );
}

#[test]
fn test_temperature_subset_download() {
    let base_url = "https://thredds.ucar.edu/thredds/dodsC/grib/NCEP/GFS/Global_0p25deg/GFS_Global_0p25deg_20231201_0000.grib2";

    let url = UrlBuilder::new(base_url)
        .add_variable("Temperature_surface")
        .add_multidimensional_constraint(
            "Temperature_surface",
            vec![
                IndexRange::Range {
                    start: 0,
                    end: 5,
                    stride: None,
                },
                IndexRange::Range {
                    start: 100,
                    end: 200,
                    stride: Some(2),
                },
                IndexRange::Range {
                    start: 300,
                    end: 400,
                    stride: None,
                },
            ],
        )
        .dods_url()
        .unwrap();

    let expected = "https://thredds.ucar.edu/thredds/dodsC/grib/NCEP/GFS/Global_0p25deg/GFS_Global_0p25deg_20231201_0000.grib2.dods?Temperature_surface[0:5][100:2:200][300:400]";
    assert_eq!(url, expected);
}

#[test]
fn test_multiple_variables_complex_constraints() {
    let base_url = "https://coastwatch.pfeg.noaa.gov/erddap/griddap/jplMURSST41";

    let time_lat_lon_constraints = vec![
        IndexRange::Range {
            start: 0,
            end: 10,
            stride: Some(2),
        },
        IndexRange::Range {
            start: 20,
            end: 50,
            stride: None,
        },
        IndexRange::Range {
            start: -180,
            end: -120,
            stride: Some(5),
        },
    ];

    let url = UrlBuilder::new(base_url)
        .add_variable("analysed_sst")
        .add_variable("analysis_error")
        .add_multidimensional_constraint("analysed_sst", time_lat_lon_constraints.clone())
        .add_multidimensional_constraint("analysis_error", time_lat_lon_constraints)
        .dods_url()
        .unwrap();

    let expected = "https://coastwatch.pfeg.noaa.gov/erddap/griddap/jplMURSST41.dods?analysed_sst[0:2:10][20:50][-180:5:-120],analysis_error[0:2:10][20:50][-180:5:-120]";
    assert_eq!(url, expected);
}

#[test]
fn test_single_point_extraction() {
    let base_url = "https://data.nodc.noaa.gov/thredds/dodsC/woa/WOA13/DATAv2/temperature/netcdf/decav/1.00/woa13_decav_t00_01v2.nc";

    let url = UrlBuilder::new(base_url)
        .add_variable("t_an")
        .add_multidimensional_constraint(
            "t_an",
            vec![
                IndexRange::Single(0),   // depth
                IndexRange::Single(90),  // lat
                IndexRange::Single(180), // lon
            ],
        )
        .dods_url()
        .unwrap();

    let expected = "https://data.nodc.noaa.gov/thredds/dodsC/woa/WOA13/DATAv2/temperature/netcdf/decav/1.00/woa13_decav_t00_01v2.nc.dods?t_an[0][90][180]";
    assert_eq!(url, expected);
}

#[test]
fn test_constraint_builder_patterns() {
    let base_url = "https://example.com/data/ocean.nc";

    let spatial_constraint = Constraint::new(
        "temperature",
        vec![
            IndexRange::Range {
                start: 0,
                end: 100,
                stride: Some(10),
            },
            IndexRange::Single(5),
            IndexRange::Range {
                start: 20,
                end: 50,
                stride: None,
            },
            IndexRange::Range {
                start: -180,
                end: 180,
                stride: Some(5),
            },
        ],
    );

    let url = UrlBuilder::new(base_url)
        .add_variable("temperature")
        .add_constraint(spatial_constraint)
        .dods_url()
        .unwrap();

    let expected =
        "https://example.com/data/ocean.nc.dods?temperature[0:10:100][5][20:50][-180:5:180]";
    assert_eq!(url, expected);
}

#[test]
fn test_builder_reuse_and_modification() {
    let base_url = "https://example.com/data/weather.nc";

    let base_builder = UrlBuilder::new(base_url).add_variable("temperature");

    let surface_url = base_builder
        .clone()
        .add_multidimensional_constraint(
            "temperature",
            vec![
                IndexRange::Range {
                    start: 0,
                    end: 10,
                    stride: None,
                },
                IndexRange::Single(0),
            ],
        )
        .dods_url()
        .unwrap();

    let upper_air_url = base_builder
        .clone()
        .add_multidimensional_constraint(
            "temperature",
            vec![
                IndexRange::Range {
                    start: 0,
                    end: 10,
                    stride: None,
                },
                IndexRange::Range {
                    start: 10,
                    end: 20,
                    stride: None,
                },
            ],
        )
        .dods_url()
        .unwrap();

    assert_eq!(
        surface_url,
        "https://example.com/data/weather.nc.dods?temperature[0:10][0]"
    );
    assert_eq!(
        upper_air_url,
        "https://example.com/data/weather.nc.dods?temperature[0:10][10:20]"
    );
}

#[test]
fn test_clear_and_rebuild() {
    let base_url = "https://example.com/data/test.nc";

    let builder = UrlBuilder::new(base_url)
        .add_variable("temp")
        .add_variable("pressure")
        .add_range("temp", 0, 10, None)
        .add_range("pressure", 5, 15, Some(2));

    let initial_url = builder.clone().dods_url().unwrap();
    assert_eq!(
        initial_url,
        "https://example.com/data/test.nc.dods?temp[0:10],pressure[5:2:15]"
    );

    let cleared_url = builder
        .clear_all()
        .add_variable("humidity")
        .add_single_index("humidity", 3)
        .dods_url()
        .unwrap();

    assert_eq!(
        cleared_url,
        "https://example.com/data/test.nc.dods?humidity[3]"
    );
}

#[test]
fn test_edge_cases() {
    let base_url = "https://example.com/data/edge_case.nc";

    let empty_url = UrlBuilder::new(base_url).dods_url().unwrap();
    assert_eq!(empty_url, "https://example.com/data/edge_case.nc.dods");

    let zero_indices = UrlBuilder::new(base_url)
        .add_variable("data")
        .add_single_index("data", 0)
        .dods_url()
        .unwrap();
    assert_eq!(
        zero_indices,
        "https://example.com/data/edge_case.nc.dods?data[0]"
    );

    let zero_range = UrlBuilder::new(base_url)
        .add_variable("data")
        .add_range("data", 0, 0, None)
        .dods_url()
        .unwrap();
    assert_eq!(
        zero_range,
        "https://example.com/data/edge_case.nc.dods?data[0:0]"
    );
}
