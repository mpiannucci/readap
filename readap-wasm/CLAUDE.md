# readap-wasm

This is a WebAssembly port of [readap](../readap), a library for parsing OpenDAP binary data. 

The goal of this crate is to build a simple OpenDAP client that can be used in the browser.

## Development

This create uses `wasm-pack` to build the WASM version of readap. It was initialized from the [wasm-pack template](https://github.com/rustwasm/wasm-pack-template) using `cargo generate --git https://github.com/rustwasm/wasm-pack-template.git --name readap-wasm`.

To build the WASM version of readap, run:

```bash
$ wasm-pack build
```

