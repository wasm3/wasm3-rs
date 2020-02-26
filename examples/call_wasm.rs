use wasm3::environment::Environment;
use wasm3::module::Module;

fn main() {
    let env = Environment::new().expect("Unable to create environment");
    let mut rt = env
        .create_runtime(1024 * 60)
        .expect("Unable to create runtime");
    let module = Module::parse(&env, &include_bytes!("wasm/wasm_add/wasm_add.wasm")[..])
        .expect("Unable to parse module");

    let module = rt.load_module(module).expect("Unable to load module");
    let func = module
        .find_function::<(i64, i64), i64>(&rt, "add")
        .expect("Unable to find function");
    println!(
        "Wasm says that 3 + 6 is {}",
        func.call(&mut rt, 3, 6).unwrap()
    )
}
