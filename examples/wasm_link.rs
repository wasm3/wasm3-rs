use wasm3::Environment;
use wasm3::Module;
use anyhow::Result;

const MILLIS: u64 = 500_000;

fn main()->Result<()> {
    let env = Environment::new()?;
    let rt = env
        .create_runtime(1024 * 60)?;
    let module = Module::parse(
        &env,
        &include_bytes!("wasm/wasm_millis_to_seconds/wasm_millis_to_seconds.wasm")[..],
    )?;

    let mut module = rt.load_module(module)?;
    module
        .link_function::<(), u64>("time", "millis", millis_wrap)?;

    let func = module
        .find_function::<(), u64>("seconds")?;

    println!("{}ms in seconds is {:?}s.", MILLIS, func.call());
    assert_eq!(func.call(), Ok(MILLIS / 1000));
    Ok(())
}

wasm3::make_func_wrapper!(millis_wrap: millis() -> u64);
fn millis() -> u64 {
    MILLIS
}
