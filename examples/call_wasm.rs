use wasm3::environment::Environment;
use wasm3::module::Module;

fn main() {
    let env = Environment::new();
    let rt = env.create_runtime(1024 * 60);
    let module = Module::parse(&env, &include_bytes!("wasm/wasm_add/wasm_add.wasm")[..]).unwrap();

    assert!(rt.load_module(module).is_ok());
    let func = rt.find_function::<(i64, i64), i64>("add").unwrap();
    println!("Wasm says that 3 + 6 is {}", func.call(3, 6).unwrap())
}
