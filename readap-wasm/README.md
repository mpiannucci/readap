# readap-wasm

WebAssembly bindings for the readap OpenDAP parser library.

## Usage

```bash
# Build
wasm-pack build --target web

# Test
wasm-pack test --headless --firefox
```

## API

- `parse_dds(content)` - Parse DDS response
- `parse_das(content)` - Parse DAS response  
- `parse_dods(bytes)` - Parse DODS binary data
- `UrlBuilder` - Build OpenDAP URLs with constraints

## License

MIT