use std::env;
use std::ffi::OsStr;
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
    let wrapper_file = out_path.join("wrapper.h");
    let header_files = [
        "wasm3.h",
        #[cfg(feature = "wasi")]
        "m3_api_wasi.h",
    ];
    let contents = header_files
        .map(|header_file| format!("#include \"{}\"\n", header_file))
        .join("");
    fs::write(&wrapper_file, contents).expect("failed to create wasm3 wrapper file");
    wrapper_file
}

#[cfg(not(feature = "build-bindgen"))]
fn gen_bindings() {
    use std::process::Command;
    let out_path = PathBuf::from(&env::var("OUT_DIR").unwrap());
    let wrapper_file = gen_wrapper(&out_path);
    let mut bindgen = Command::new("bindgen");
    bindgen.arg(wrapper_file);
    bindgen.args([
        "--use-core",
        "--ctypes-prefix",
        "cty",
        "--no-layout-tests",
        "--default-enum-style=moduleconsts",
        "--no-doc-comments",
        "--allowlist-function",
        WHITELIST_REGEX_FUNCTION,
        "--allowlist-type",
        WHITELIST_REGEX_TYPE,
        "--allowlist-var",
        WHITELIST_REGEX_VAR,
        "--no-derive-debug",
    ]);
    for &ty in PRIMITIVES {
        bindgen.arg("--blocklist-type").arg(ty);
    }
    bindgen
        .arg("-o")
        .arg(out_path.join("bindings.rs").to_str().unwrap());
    bindgen
        .arg("--")
        .arg(format!(
            "-Dd_m3Use32BitSlots={}",
            cfg!(feature = "use-32bit-slots") as u8,
        ))
        .arg("-Dd_m3LogOutput=0")
        .arg("-Iwasm3/source");
    let status = match bindgen.status() {
        Ok(status) => status,
        Err(error) => panic!("wasm3: unable to generate bindings: {error}"),
    };
    if !status.success() {
        panic!("wasm3: failed to run bindgen: {:?}", status);
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
        .allowlist_function(WHITELIST_REGEX_FUNCTION)
        .allowlist_type(WHITELIST_REGEX_TYPE)
        .allowlist_var(WHITELIST_REGEX_VAR)
        .derive_debug(false);
    bindgen = PRIMITIVES
        .iter()
        .fold(bindgen, |bindgen, ty| bindgen.blocklist_type(ty));
    let bindgen = bindgen.clang_args(
        [
            &format!(
                "-Dd_m3Use32BitSlots={}",
                cfg!(feature = "use-32bit-slots") as u8,
            ),
            "-Dd_m3LogOutput=0",
            "-Iwasm3/source",
        ]
        .iter(),
    );
    bindgen
        .generate()
        .unwrap_or_else(|error| panic!("bindgen: failed to generate bindings: {error}"))
        .write_to_file(out_path.join("bindings.rs").to_str().unwrap())
        .unwrap_or_else(|error| panic!("bindgen: failed to write bindings: {error}"));
}

fn main() {
    gen_bindings();

    let mut cfg = cc::Build::new();

    cfg.files(
        fs::read_dir(WASM3_SOURCE)
            .unwrap_or_else(|error| panic!("failed to read {WASM3_SOURCE} directory: {error}"))
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|p| p.extension().and_then(OsStr::to_str) == Some("c")),
    );

    cfg.cpp(false)
        .define("d_m3LogOutput", Some("0"))
        .warnings(false)
        .extra_warnings(false)
        .include(WASM3_SOURCE);

    // Add any extra arguments from the environment to the CC command line.
    if let Ok(extra_clang_args) = std::env::var("BINDGEN_EXTRA_CLANG_ARGS") {
        // Try to parse it with shell quoting. If we fail, make it one single big argument.
        if let Some(strings) = shlex::split(&extra_clang_args) {
            strings.iter().for_each(|string| {
                cfg.flag(string);
            })
        } else {
            cfg.flag(&extra_clang_args);
        };
    }

    if cfg!(feature = "wasi") {
        cfg.define("d_m3HasWASI", None);
    }

    cfg.define(
        "d_m3Use32BitSlots",
        match cfg!(feature = "use-32bit-slots") {
            true => Some("1"),
            false => Some("0"),
        }
    );
    cfg.compile("wasm3");
}
