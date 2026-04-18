[group("wasm")]
@wasm-build *args:
    wasm-pack build mcpixel --features wasm {{args}}

[group("wasm")]
@wasm-clippy *args:
    cargo clippy -p mcpixel --target wasm32-unknown-unknown --features wasm {{args}}

[group("wasm")]
@web:
    cd mcpixel-web && bun run dev --open