use wasm3::environment::Environment;
use wasm3::module::Module;
use wasm3::runtime::Runtime;

fn runtime() -> Runtime {
    Environment::new()
        .expect("Unable to create environment")
        .create_runtime(1024 * 60)
        .expect("Unable to create runtime")
}

fn module(rt: &mut Runtime) -> Module {
    rt.parse_and_load_module(&include_bytes!("wasm_test_bins/wasm_test_bins.wasm")[..])
        .expect("Unable to load module")
}

#[test]
fn test_binary_func() {
    let rt = &mut runtime();
    let module = module(rt);
    let func = module
        .find_function::<(u64, u64), u64>(rt, "add")
        .expect("Unable to find function");
    assert_eq!(func.call(rt, 124, 612), Ok(736));
}

#[test]
fn test_unary_func() {
    let rt = &mut runtime();
    let module = module(rt);
    let func = module
        .find_function::<u64, u64>(rt, "invert")
        .expect("Unable to find function");
    assert_eq!(func.call(rt, 736), Ok(!736));
}

#[test]
fn test_no_return_func() {
    let rt = &mut runtime();
    let module = module(rt);
    let func = module
        .find_function::<u64, ()>(rt, "no_return")
        .expect("Unable to find function");
    assert_eq!(func.call(rt, 736), Ok(()));
}

#[test]
fn test_no_args_func() {
    let rt = &mut runtime();
    let module = module(rt);
    let func = module
        .find_function::<(), u64>(rt, "constant")
        .expect("Unable to find function");
    assert_eq!(func.call(rt), Ok(0xDEAD_BEEF_0000_FFFF));
}

#[test]
fn test_no_args_u32_func() {
    let rt = &mut runtime();
    let module = module(rt);
    let func = module
        .find_function::<(), u32>(rt, "u32")
        .expect("Unable to find function");
    assert_eq!(func.call(rt), Ok(0xDEAD_BEEF));
}

#[test]
fn test_no_args_no_ret_func() {
    let rt = &mut runtime();
    let module = module(rt);
    let func = module
        .find_function::<(), ()>(rt, "empty")
        .expect("Unable to find function");
    assert_eq!(func.call(rt), Ok(()));
}
