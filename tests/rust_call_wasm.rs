use wasm3::Environment;
use wasm3::Module;
use wasm3::Runtime;

fn runtime() -> Runtime {
    Environment::new()
        .expect("Unable to create environment")
        .create_runtime(1024 * 60)
        .expect("Unable to create runtime")
}

fn module(rt: &Runtime) -> Module {
    rt.parse_and_load_module(&include_bytes!("wasm_test_bins/wasm_test_bins.wasm")[..])
        .expect("Unable to load module")
}

#[test]
fn test_add_u64() {
    let rt = runtime();
    let module = module(&rt);
    let func = module
        .find_function::<(u64, u64), u64>("add_u64")
        .expect("Unable to find function");
    assert_eq!(func.call(124, 612), Ok(736));
}

#[test]
fn test_add_u32() {
    let rt = runtime();
    let module = module(&rt);
    let func = module
        .find_function::<(u32, u32), u32>("add_u32")
        .expect("Unable to find function");
    assert_eq!(func.call(124, 612), Ok(736));
}

#[test]
fn test_unary_func() {
    let rt = runtime();
    let module = module(&rt);
    let func = module
        .find_function::<u64, u64>("invert")
        .expect("Unable to find function");
    assert_eq!(func.call(736), Ok(!736));
}

#[test]
fn test_no_return_func() {
    let rt = runtime();
    let module = module(&rt);
    let func = module
        .find_function::<u64, ()>("no_return")
        .expect("Unable to find function");
    assert_eq!(func.call(736), Ok(()));
}

#[test]
fn test_no_args_func() {
    let rt = runtime();
    let module = module(&rt);
    let func = module
        .find_function::<(), u64>("constant")
        .expect("Unable to find function");
    assert_eq!(func.call(), Ok(0xDEAD_BEEF_0000_FFFF));
}

#[test]
fn test_no_args_u32_func() {
    let rt = runtime();
    let module = module(&rt);
    let func = module
        .find_function::<(), u32>("u32")
        .expect("Unable to find function");
    assert_eq!(func.call(), Ok(0xDEAD_BEEF));
}

#[test]
fn test_no_args_no_ret_func() {
    let rt = runtime();
    let module = module(&rt);
    let func = module
        .find_function::<(), ()>("empty")
        .expect("Unable to find function");
    assert_eq!(func.call(), Ok(()));
}
