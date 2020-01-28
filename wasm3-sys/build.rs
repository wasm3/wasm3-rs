use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{BufWriter, Result, Write};
use std::path::PathBuf;

fn gen_bindings() -> Result<()> {
    let whitelist_regex =
        "((?:I|c_)?[Mm]3.*)|.*Page.*|Module_.*|EmitWord_impl|op_CallRawFunction|Compile_Function";

    let root_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let wrapper_file = root_path.join("wrapper.h");
    let wrapper_file = wrapper_file.to_str().unwrap();

    {
        let file = fs::File::create(wrapper_file).expect("failed to create wasm3 wrapper file");
        let mut file = BufWriter::new(file);
        for path in fs::read_dir("wasm3/source")
            .expect("failed to read wasm3/source directory")
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(OsStr::to_str) == Some("h"))
        {
            writeln!(file, "#include \"{}\"", path.to_str().unwrap()).unwrap();
        }
    }

    let mut bindgen = std::process::Command::new("bindgen");

    let bindgen = bindgen
        .arg(wrapper_file)
        .arg("--use-core")
        .arg("--ctypes-prefix")
        .arg("cty")
        .arg("--no-layout-tests")
        .arg("--default-enum-style=moduleconsts")
        .arg("--no-doc-comments")
        .arg("--whitelist-function")
        .arg(whitelist_regex)
        .arg("--whitelist-type")
        .arg(whitelist_regex)
        .arg("--whitelist-var")
        .arg(whitelist_regex)
        .arg("--no-derive-debug");

    let bindgen = [
        "f64", "f32", "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8",
    ]
    .iter()
    .fold(bindgen, |bindgen, &ty| {
        bindgen.arg("--blacklist-type").arg(ty)
    });

    let status = bindgen
        .arg("-o")
        .arg(out_path.join("bindings.rs").to_str().unwrap())
        .status()
        .expect("Unable to generate bindings");

    if !status.success() {
        panic!("Failed to run bindgen: {:?}", status);
    }

    Ok(())
}

fn main() -> Result<()> {
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
    let cfg = cfg
        .warnings(false)
        .cpp(false)
        .define("d_m3LogOutput", Some("0"))
        .extra_warnings(false)
        .include("wasm3/source");
    if cfg!(feature = "wasi") {
        cfg.define("d_m3HasWASI", None);
    }
    cfg.compile("wasm3");
    Ok(())
}
