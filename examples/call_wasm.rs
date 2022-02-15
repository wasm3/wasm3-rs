use wasm3::Environment;
use wasm3::Module;
use anyhow::Result;

fn main()->Result<()> {
    let env = Environment::new()?;
    let rt = env
        .create_runtime(1024 * 60)?;
    let module = Module::parse(&env, &include_bytes!("wasm/wasm_add/wasm_add.wasm")[..])?;

    let module = rt.load_module(module)?;
    let func = module
        .find_function::<(i64, i64), i64>("add")?;
    println!("Wasm says that 3 + 6 is {}", func.call(3, 6)?);
    Ok(())
}
