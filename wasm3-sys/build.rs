use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::PathBuf;

fn gen_bindings() -> io::Result<()> {
    const PRIMITIVES: &[&str] = &[
        "f64", "f32", "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8",
    ];
    let whitelist_regex = "(I|c_)?[Mm]3.*";
    let mut bindgen = bindgen::builder()
        .layout_tests(false)
        .generate_comments(false)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .whitelist_function(whitelist_regex)
        .whitelist_type(whitelist_regex)
        .whitelist_var(whitelist_regex)
        .derive_debug(false);
    for path in fs::read_dir("wasm3/source")?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
    {
        if path.extension().and_then(OsStr::to_str) == Some("h") {
            bindgen = bindgen.header(path.to_str().unwrap());
        }
    }
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut bindings = bindgen
        .generate()
        .expect("Unable to generate bindings")
        .to_string();
    // get rid of the `pub type i64 = i64;` cyclid definitions
    for prim in PRIMITIVES.iter() {
        let tdef = &format!("pub type {0} = {0};", prim);
        if let Some(pos) = bindings.find(tdef) {
            bindings.replace_range(pos..(pos + tdef.len()), "");
        }
    }
    std::fs::write(out_path.join("bindings.rs"), bindings.as_bytes())
}

fn main() -> io::Result<()> {
    gen_bindings()?;
    // build
    let mut cfg = cc::Build::new();
    cfg.files(
        fs::read_dir("wasm3/source")?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|p| p.extension().and_then(OsStr::to_str) == Some("c")),
    );
    if env::var_os("PROFILE") == Some(OsString::from("release"))
        && cfg.get_compiler().is_like_msvc()
    {
        cfg.flag("/Oxs");
        cfg.flag("/Oy");
        cfg.flag("/GS-");
        cfg.flag("/Zo");
        cfg.flag("/arch:AVX2");
    }
    cfg.warnings(false)
        .cpp(false)
        .define("d_m3LogOutput", Some("0"))
        .extra_warnings(false)
        .include("wasm3/source")
        .compile("wasm3");
    Ok(())
}
