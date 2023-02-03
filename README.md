# readap

Read OpenDAP binary data with pure rust

## Install

with `cargo add`: 

```bash
cargo add --git https://github.com/mpiannucci/readap
```

or `Cargo.toml`: 

```toml
[dependencies]
readap = { git = "https://github.com/mpiannucci/readap" }
```

## Getting Started

#### Read DAS metadata

```rs
let attrs = parse_das_attributes(input).unwrap();
```

This returns a `HashMap` containing all of the attributes and their children, within another hashmap

```rs
let units: String = attrs["time"]["units"].clone().try_into().unwrap();
```

### Read a DDS dataset

```rs
let dataset = DdsDataset::from_bytes(&input).unwrap();
```

### Read a DODS dataset

```rs
let dataset = DodsDataset::from_bytes(&input).unwrap();
```

Then extract the data and coordinates for a given variable

```rs
let mwd = if let DataArray::Int32(mwd) = dataset.variable_data("mean_wave_dir").unwrap() {
   	mwd
} else {
    vec![]
};

let coords = dataset.variable_coords("mean_wave_dir").unwrap();
let time_data: Vec<i32> = coords[0].1.try_into().unwrap();
```

For concrete examples, see the [parse tests](tests/parse.rs)

## What this library is

This library is an OpenDAP binary data and metadata parser. It *is not* a data downloader. It is a lower level tool to be used in higher level data access applications. 

## License

[MIT](LICENSE) - 2023 Matthew Iannucci
