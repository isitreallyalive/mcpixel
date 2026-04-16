[group("wasm")]
@wasm-build *args:
    wasm-pack build mcpixel --features wasm {{args}}