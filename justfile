cargo := env_var_or_default("CARGO", "cargo")

build: build-components

build-components:
    {{cargo}} build --manifest-path components/http-fs-hello/Cargo.toml --target=wasm32-wasip2

test: test-int

test-int: build-components
    {{cargo}} test int -- --nocapture
