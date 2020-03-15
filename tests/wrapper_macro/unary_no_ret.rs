wasm3::make_func_wrapper!(unary_wrap: unary(foo: u64));
fn unary(foo: u64) {
    let _ = foo;
}

fn main() {}
