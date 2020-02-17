# wasm3-rs ![Build](https://github.com/Veykril/wasm3-rs/workflows/Rust/badge.svg?branch=master) ![Tokei](https://tokei.rs/b1/github/veykril/wasm3-rs)

Rust wrapper for [WASM3](https://github.com/wasm3/wasm3).

This is currently work in progress and may or may not be sound.

## Usage

The crate has yet to be published on crates.io(there are a few things I wanna get done before publishing still), due to this one currently has to specify the crate as a git dependency:

```toml
[dependencies]
wasm3 = { git = "https://github.com/Veykril/wasm3-rs.git" }
```

## Sample

A simple [example](./examples/call_wasm.rs) that loads a wasm module and executes an exported function to add two `i64`s together.

```rust
use wasm3::environment::Environment;
use wasm3::module::Module;

fn main() {
    let env = Environment::new().expect("Unable to create environment");
    let rt = env
        .create_runtime(1024 * 60)
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
It does however require [Clang 9](https://releases.llvm.org/download.html#9.0.0) to be installed as well as [Bindgen](https://github.com/rust-lang/rust-bindgen).

## License
Licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)