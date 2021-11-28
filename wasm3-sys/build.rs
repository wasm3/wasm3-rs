use std::{env, ffi::OsStr, fs, path::PathBuf};

static WASM3_SOURCE: &str = "wasm3/source";
const WHITELIST_REGEX_FUNCTION: &str = "([A-Z]|m3_).*";
const WHITELIST_REGEX_TYPE: &str = "(?:I|c_)?[Mm]3.*";
const WHITELIST_REGEX_VAR: &str = WHITELIST_REGEX_TYPE;
const PRIMITIVES: &[&str] = &[
    "f64", "f32", "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8",
];

fn gen_bindings() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let mut bindgen = bindgen::builder()
        .header("wasm3/source/wasm3.h")
        .header("wasm3/source/m3_env.h")
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
                &format!(
                    "-Dd_m3HasFloat={}",
                    if cfg!(feature = "floats") { 1 } else { 0 }
                ),
                "-Dd_m3VerboseErrorMessages=0",
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

    let mut cfg = cc::Build::new();

    cfg.files(
        fs::read_dir(WASM3_SOURCE)
            .unwrap_or_else(|_| panic!("failed to read {} directory", WASM3_SOURCE))
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|p| p.extension().and_then(OsStr::to_str) == Some("c")),
    );

    cfg.cpp(false)
        .warnings(false)
        .extra_warnings(false)
        .include(WASM3_SOURCE);

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap();

    // Set options specific for x86_64-fortanix-unknown-sgx target.
    if target_arch == "x86_64" && target_env == "sgx" && target_vendor == "fortanix" {
        // Disable the stack protector as the Fortanix ABI currently sets FS and GS bases to the
        // same value and the stack protector assumes the canary value is at FS:0x28 but with the
        // Fortanix ABI that contains a copy of RSP.
        cfg.flag("-fno-stack-protector");
    }

    // Add any extra arguments from the environment to the CC command line.
    if let Ok(extra_clang_args) = env::var("BINDGEN_EXTRA_CLANG_ARGS") {
        // Try to parse it with shell quoting. If we fail, make it one single big argument.
        if let Some(strings) = shlex::split(&extra_clang_args) {
            strings.iter().for_each(|string| {
                cfg.flag(string);
            })
        } else {
            cfg.flag(&extra_clang_args);
        };
    }

    cfg.define(
        "d_m3Use32BitSlots",
        if cfg!(feature = "use-32bit-slots") {
            Some("1")
        } else {
            Some("0")
        },
    );
    cfg.define(
        "d_m3HasFloat",
        if cfg!(feature = "floats") {
            Some("1")
        } else {
            Some("0")
        },
    );
    cfg.define("d_m3VerboseErrorMessages", Some("0"));
    cfg.compile("wasm3");
}
