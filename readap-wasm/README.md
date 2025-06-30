# readap-wasm

**WebAssembly bindings for the readap OpenDAP parser library**

A WebAssembly wrapper around the [readap](../readap) Rust library, enabling OpenDAP data parsing in web browsers and Node.js environments.

## About

This package provides WebAssembly bindings for the readap library, allowing you to parse OpenDAP binary data and metadata directly in JavaScript/TypeScript applications. It supports parsing DAS (Data Attribute Structure), DDS (Data Descriptor Structure), and DODS (Data Object) formats.

## Installation

Install from NPM (once published):

```bash
npm install readap-wasm
```

Or use in a web page:

```html
<script type="module">
  import init, { greet } from './pkg/readap_wasm.js';
  
  async function run() {
    await init();
    greet();
  }
  
  run();
</script>
```

## Development

### Build the WebAssembly package

```bash
wasm-pack build
```

### Test in headless browsers

```bash
wasm-pack test --headless --firefox
```

### Publish to NPM

```bash
wasm-pack publish
```

## Usage

Once the WASM bindings are implemented, you'll be able to parse OpenDAP data in JavaScript:

**TODO**

## Features

**TODO**
* Full WebAssembly compatibility for browser and Node.js environments
* Built with [`wasm-bindgen`](https://github.com/rustwasm/wasm-bindgen) for seamless JavaScript integration
* Includes [`console_error_panic_hook`](https://github.com/rustwasm/console_error_panic_hook) for better debugging

## License

[MIT](LICENSE)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
