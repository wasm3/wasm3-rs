use wasm3::Environment;
use wasm3::Module;
use anyhow::Result;

#[cfg(feature = "wasi")]
fn main()->Result<()> {
    let env = Environment::new()?;
    let rt = env
        .create_runtime(1024 * 60)?;

    let module = Module::parse(&env, &include_bytes!("wasm/wasm_print/wasm_print.wasm")[..])?;
    let mut module = rt.load_module(module)?;

    module.link_wasi()?;

    let func = module
        .find_function::<(), ()>("_start")?;

    func.call()?;

    Ok(())
}

#[cfg(not(feature = "wasi"))]
fn main() {
    panic!("This example requires the wasi feature");
}
