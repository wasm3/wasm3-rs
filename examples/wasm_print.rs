use wasm3::environment::Environment;
use wasm3::module::Module;

#[cfg(feature = "wasi")]
fn main() {
    let env = Environment::new().expect("Unable to create environment");
    let rt = env
        .create_runtime(1024 * 60)
        .expect("Unable to create runtime");
    let module = Module::parse(&env, &include_bytes!("wasm/wasm_print/wasm_print.wasm")[..])
        .expect("Unable to parse module");

    let mut module = rt.load_module(module).expect("Unable to load module");
    module.link_wasi().expect("Failed to link wasi");
    let func = module
        .find_function::<(), ()>("print_hello_world")
        .expect("Unable to find function");
    func.call().unwrap();
}

#[cfg(not(feature = "wasi"))]
fn main() {
    panic!("This example requires the wasi feature");
}
