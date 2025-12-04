# `http-fs-hello`, with wasmCloud v2

This repository contains a WebAssembly component built using recent Rust ecosystem tooling
like [`wstd`][wstd] to serve HTTP requests via [wasmCloud v2][wasmcloud-v2] (i.e. `wash`
and `wash-runtime`).

[wstd]: https://github.com/bytecodealliance/wstd
[wasmcloud-v2]: https://github.com/wasmcloud/wash

# Quickstart

To build just the component:

```console
just build
```

To run the integration tests which start a custom wasmCloud runtime, and a
workload that contains the components:

```console
RUST_LOG=debug just test-int
```
