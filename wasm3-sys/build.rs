use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::PathBuf;

fn gen_bindings() -> io::Result<()> {
    let whitelist_regex =
        "((?:I|c_)?[Mm]3.*)|(.*Page.*)|(Module_.*)|EmitWord_impl|op_CallRawFunction";
    let bindgen = bindgen::builder()
        .layout_tests(false)
        .generate_comments(false)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .whitelist_function(whitelist_regex)
        .whitelist_type(whitelist_regex)
        .whitelist_var(whitelist_regex)
        .derive_debug(false);
    let bindgen = fs::read_dir("wasm3/source")?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(OsStr::to_str) == Some("h"))
        .fold(bindgen, |bindgen, path| {
            bindgen.header(path.to_str().unwrap())
        });
    let bindgen = [
        "f64", "f32", "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8",
    ]
    .iter()
    .fold(bindgen, |bindgen, &ty| bindgen.blacklist_type(ty));

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
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
