use std::env;
use std::ffi::OsStr;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

static WASM3_SOURCE: &str = "wasm3/source";
const WHITELIST_REGEX_FUNCTION: &str = "([A-Z]|m3_).*";
const WHITELIST_REGEX_TYPE: &str = "(?:I|c_)?[Mm]3.*";
const WHITELIST_REGEX_VAR: &str = WHITELIST_REGEX_TYPE;
const PRIMITIVES: &[&str] = &[
    "f64", "f32", "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8",
];

fn gen_wrapper(out_path: &Path) -> PathBuf {
    // TODO: we currently need the field definitions of the structs of wasm3. These aren't exposed
    // in the wasm3.h header so we have to generate bindings for more.
    let wrapper_file = out_path.join("wrapper.h");

    let mut buffer = String::new();
    fs::read_dir(WASM3_SOURCE)
        .unwrap_or_else(|_| panic!("failed to read {} directory", WASM3_SOURCE))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(OsStr::to_str) == Some("h"))
        .for_each(|path| {
            writeln!(
                &mut buffer,
                "#include \"{}\"",
                path.file_name().unwrap().to_str().unwrap()
            )
            .unwrap()
        });
    fs::write(&wrapper_file, buffer).expect("failed to create wasm3 wrapper file");
    wrapper_file
}

#[cfg(not(feature = "build-bindgen"))]
fn gen_bindings() {
    let out_path = PathBuf::from(&env::var("OUT_DIR").unwrap());

    let wrapper_file = gen_wrapper(&out_path);

    let mut bindgen = std::process::Command::new("bindgen");
    bindgen
        .arg(wrapper_file)
        .arg("--use-core")
        .arg("--ctypes-prefix")
        .arg("cty")
        .arg("--no-layout-tests")
        .arg("--default-enum-style=moduleconsts")
        .arg("--no-doc-comments")
        .arg("--whitelist-function")
        .arg(WHITELIST_REGEX_FUNCTION)
        .arg("--whitelist-type")
        .arg(WHITELIST_REGEX_TYPE)
        .arg("--whitelist-var")
        .arg(WHITELIST_REGEX_VAR)
        .arg("--no-derive-debug");
    for &ty in PRIMITIVES.iter() {
        bindgen.arg("--blacklist-type").arg(ty);
    }
    bindgen
        .arg("-o")
        .arg(out_path.join("bindings.rs").to_str().unwrap());
    bindgen
        .arg("--")
        .arg(format!(
            "-Dd_m3Use32BitSlots={}",
            if cfg!(feature = "use-32bit-slots") {
                1
            } else {
                0
            }
        ))
        .arg("-Dd_m3LogOutput=0")
        .arg("-Iwasm3/source");
    let status = bindgen.status().expect("Unable to generate bindings");
    if !status.success() {
        panic!("Failed to run bindgen: {:?}", status);
    }
}

#[cfg(feature = "build-bindgen")]
fn gen_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let wrapper_file = gen_wrapper(&out_path);

    let mut bindgen = bindgen::builder()
        .header(wrapper_file.to_str().unwrap())
        .use_core()
        .ctypes_prefix("cty")
        .layout_tests(false)
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .generate_comments(false)
        .whitelist_function(WHITELIST_REGEX_FUNCTION)
        .whitelist_type(WHITELIST_REGEX_TYPE)
        .whitelist_var(WHITELIST_REGEX_VAR)
        .derive_debug(false);
    bindgen = PRIMITIVES
        .iter()
        .fold(bindgen, |bindgen, ty| bindgen.blacklist_type(ty));
    bindgen
        .clang_args(
            [
                &format!(
                    "-Dd_m3Use32BitSlots={}",
                    if cfg!(feature = "use-32bit-slots") {
                        1
                    } else {
                        0
                    }
                ),
                "-Dd_m3LogOutput=0",
                "-Iwasm3/source",
            ]
            .iter(),
        )
        .generate()
        .expect("Failed to generate bindings")
        .write_to_file(out_path.join("bindings.rs").to_str().unwrap())
        .expect("Failed to write bindings");
}

fn main() {
    gen_bindings();
    // build
    let mut cfg = cc::Build::new();
    cfg.files(
        fs::read_dir(WASM3_SOURCE)
            .unwrap_or_else(|_| panic!("failed to read {} directory", WASM3_SOURCE))
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|p| p.extension().and_then(OsStr::to_str) == Some("c")),
    );
    let cfg = cfg
        .warnings(false)
        .cpp(false)
        .define("d_m3LogOutput", Some("0"))
        .extra_warnings(false)
        .include(WASM3_SOURCE);
    if cfg!(feature = "wasi") {
        cfg.define("d_m3HasWASI", None);
    }
    cfg.define(
        "d_m3Use32BitSlots",
        if cfg!(feature = "use-32bit-slots") {
            Some("1")
        } else {
            Some("0")
        },
    );
    cfg.compile("wasm3");
}
