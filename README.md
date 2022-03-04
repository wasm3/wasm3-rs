[![SWUbanner](https://raw.githubusercontent.com/vshymanskyy/StandWithUkraine/main/banner-direct.svg)](https://github.com/vshymanskyy/StandWithUkraine/blob/main/docs/README.md)

# wasm3-rs

![Build](https://github.com/wasm3/wasm3-rs/workflows/Rust/badge.svg?branch=master)
[![Crates.io](https://img.shields.io/crates/v/wasm3.svg)](https://crates.io/crates/wasm3)
[![Docs.rs](https://docs.rs/wasm3/badge.svg)](https://docs.rs/wasm3)
![Tokei](https://tokei.rs/b1/github/wasm3/wasm3-rs)

## > wasm3-rs is looking for a maintainer <

Rust wrapper for [WASM3](https://github.com/wasm3/wasm3).

This is currently work in progress and may or may not be entirely sound.

## Sample

A simple [example](./examples/call_wasm.rs) that loads a wasm module and executes an exported function to add two `i64`s together.

```rust
use wasm3::Environment;
use wasm3::Module;

fn main() {
    let env = Environment::new().expect("Unable to create environment");
    let rt = env
        .create_runtime(1024)
        .expect("Unable to create runtime");
    let module = Module::parse(&env, &include_bytes!("wasm/wasm_add/wasm_add.wasm")[..])
        .expect("Unable to parse module");

    let module = rt.load_module(module).expect("Unable to load module");
    let func = module
        .find_function::<(i64, i64), i64>("add")
        .expect("Unable to find function");
    println!("Wasm says that 3 + 6 is {}", func.call(3, 6).unwrap())
}
```

## Building

This crate currently does not make use of the cmake project of wasm3, meaning cmake is not required to built this for the time being.
It does however require [Clang 9](https://releases.llvm.org/download.html#9.0.0) to be installed as well as [Bindgen](https://github.com/rust-lang/rust-bindgen), should the `build-bindgen` feature not be set.

The wasm3 c source is included via a submodule, so before building the submodule has to be initialized, this can be done via:
```sh
git submodule update --init
```

Then to build the project run:

```sh
cargo install bindgen
cargo build --release

# or:
cargo build --release --features build-bindgen

# or, enable only specific features:
cargo build --release --no-default-features --features build-bindgen,std,use-32bit-slots,wasi
```


## Build and run examples

```sh
rustup target add wasm32-unknown-unknown

python wasm_bin_builder.py ./examples/wasm/wasm_add
cargo run --example call_wasm

python wasm_bin_builder.py ./examples/wasm/wasm_print
cargo run --example wasm_print
```


## License

Licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
