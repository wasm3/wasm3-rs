wasm3::make_func_wrapper!(unary_wrap: unary(foo: u64) -> u64);
fn unary(foo: u64) -> u64 {
    foo
}

fn main() {}
