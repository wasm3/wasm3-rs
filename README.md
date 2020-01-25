# wasm3-rs

Rust wrapper for [WASM3](https://github.com/wasm3/wasm3).

This is currently very much work in progress and may or may not be sound.

## Sample

A simple [example](./examples/call_wasm.rs) that loads a wasm module and executes an exported function to add two `i64`s together.

```rust
use wasm3::environment::Environment;
use wasm3::module::Module;

fn main() {
    let env = Environment::new();
    let rt = env.create_runtime(1024 * 60);
    let data = include_bytes!("wasm/wasm-add/wasm_add.wasm");
    let module = Module::parse(&env, &data[..]).unwrap();

    assert!(rt.load_module(module).is_ok());
    let func = rt.find_function::<(i64, i64), i64>("add").unwrap();
    println!("Wasm says that 3 + 6 is {}", func.call(3, 6).unwrap())
}
```

## Building
This crate currently does not make use of the cmake project of wasm3, meaning cmake is not required to built this for the time being.
It does however require [Clang 9](https://releases.llvm.org/download.html#9.0.0) to be installed.

## License
Licensed under the MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)