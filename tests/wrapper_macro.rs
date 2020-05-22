#[test]
#[ignore]
fn wrapper_macro() {
    let t = trybuild::TestCases::new();
    t.pass("tests/wrapper_macro/*.rs");
}
