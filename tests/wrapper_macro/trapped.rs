use wasm3::error::TrappedResult;

wasm3::make_func_wrapper!(unary_wrap: unary(foo: u64) -> TrappedResult<u64>);
fn unary(foo: u64) -> TrappedResult<u64> {
    Ok(foo)
}

fn main() {}
