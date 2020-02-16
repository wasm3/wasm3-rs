use wasm3::environment::Environment;
use wasm3::module::Module;

const MILLIS: u64 = 500_000;

fn main() {
    let env = Environment::new();
    let rt = env.create_runtime(1024 * 60);
    let module = Module::parse(
        &env,
        &include_bytes!("wasm/wasm_millis_to_seconds/wasm_millis_to_seconds.wasm")[..],
    )
    .unwrap();

    let mut module = rt.load_module(module).map_err(|(_, e)| e).unwrap();
    module.link_closure("time", "millis", |()| MILLIS).unwrap();
    let func = module.find_function::<(), u64>("seconds").unwrap();
    println!("{}ms in seconds is {:?}s.", MILLIS, func.call());
    assert_eq!(func.call(), Ok(MILLIS / 1000));
}
