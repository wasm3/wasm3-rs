use wasm3::error::Error;
use wasm3::error::Trap;
use wasm3::Environment;
use wasm3::Module;
use anyhow::Result;

fn main()->Result<()> {
    let env = Environment::new()?;
    let rt = env
        .create_runtime(1024 * 60)?;
    let module = Module::parse(
        &env,
        &include_bytes!("wasm/wasm_millis_to_seconds/wasm_millis_to_seconds.wasm")[..],
    )?;

    let mut module = rt.load_module(module).expect("Unable to load module");
    module
        .link_closure("time", "millis", |_, ()| Err::<u64, _>(Trap::Abort))?;

    let func = module
        .find_function::<(), u64>("seconds")?;

    let err = func.call().unwrap_err();
    match err {
        Error::Wasm3(e) if e.is_trap(Trap::Abort) => {
            println!("got expected error: {}", e);
        }
        _ => {
            panic!("unexpected error: {}", err)
        }
    }
    Ok(())
}
