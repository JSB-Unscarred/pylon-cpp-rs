//! Compiles the C shim and links it against pylon. With the `bindgen` feature it first regenerates
//! src/bindings.rs.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=shim");

    #[cfg(feature = "bindgen")]
    generate_bindings();

    // Build scripts run on the host, so the target comes from Cargo rather than from cfg.
    let (cflags, links) = match env::var("CARGO_CFG_TARGET_OS").unwrap().as_str() {
        "windows" => windows(),
        "linux" => linux(),
        "macos" => macos(),
        os => panic!("pylon is not supported on {os}"),
    };

    let mut build = cc::Build::new();
    build.cpp(true).std("c++17").file("shim/pylon_shim.cpp");
    for flag in cflags {
        build.flag(flag);
    }
    build.compile("pylon_shim");

    // Printed after the shim library so that linkers resolving in order find pylon after it.
    for link in links {
        println!("cargo::{link}");
    }
}

/// Compiler flags for the shim and link instructions for pylon.
type Config = (Vec<String>, Vec<String>);

fn windows() -> Config {
    println!("cargo::rerun-if-env-changed=PYLON_DEV_DIR");
    let dir = PathBuf::from(env::var("PYLON_DEV_DIR").expect("PYLON_DEV_DIR is not set"));
    (
        // The pylon headers select their import libraries through #pragma comment(lib).
        vec![format!("-I{}", dir.join("include").display()), "/EHsc".into()],
        vec![format!("rustc-link-search=native={}", dir.join("lib").join("x64").display())],
    )
}

fn linux() -> Config {
    println!("cargo::rerun-if-env-changed=PYLON_ROOT");
    let root = env::var("PYLON_ROOT").unwrap_or_else(|_| "/opt/pylon".into());
    let pylon_config = |arg| {
        let tool = PathBuf::from(&root).join("bin").join("pylon-config");
        let output = Command::new(&tool).arg(arg).output();
        let output = output.unwrap_or_else(|e| panic!("{}: {e}", tool.display()));
        assert!(output.status.success(), "{} {arg} failed", tool.display());
        String::from_utf8(output.stdout).unwrap()
    };
    let cflags = pylon_config("--cflags").split_whitespace().map(String::from).collect();
    // Other flags such as -Wl,... cannot reach the final binary through Cargo.
    let links = pylon_config("--libs")
        .split_whitespace()
        .filter_map(|flag| {
            if let Some(dir) = flag.strip_prefix("-L") {
                Some(format!("rustc-link-search=native={dir}"))
            } else {
                flag.strip_prefix("-l").map(|lib| format!("rustc-link-lib={lib}"))
            }
        })
        .collect();
    (cflags, links)
}

fn macos() -> Config {
    println!("cargo::rerun-if-env-changed=PYLON_ROOT");
    let root = PathBuf::from(
        env::var("PYLON_ROOT").unwrap_or_else(|_| "/Library/Frameworks/pylon.framework".into()),
    );
    let frameworks = root.parent().expect("PYLON_ROOT is a framework path").display();
    (
        vec![
            format!("-F{frameworks}"),
            format!("-I{}", root.join("Headers").join("GenICam").display()),
        ],
        vec![
            format!("rustc-link-search=framework={frameworks}"),
            "rustc-link-lib=framework=pylon".into(),
        ],
    )
}

/// Writes src/bindings.rs. The header has the same ABI on every supported target, so the
/// bindings are generated for each of them and must be identical.
#[cfg(feature = "bindgen")]
fn generate_bindings() {
    const TARGETS: [&str; 5] = [
        "x86_64-pc-windows-msvc",
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
        "x86_64-apple-darwin",
        "aarch64-apple-darwin",
    ];
    let generate = |target: &str| {
        bindgen::Builder::default()
            .header("shim/pylon_shim.h")
            // Freestanding: only the compiler's own stdint/stddef/stdbool, no target system headers.
            .clang_args([format!("--target={target}"), "-ffreestanding".into()])
            .allowlist_file(".*pylon_shim\\.h")
            .default_enum_style(bindgen::EnumVariation::NewType {
                is_bitfield: false,
                is_global: false,
            })
            .derive_default(true)
            .rust_edition(bindgen::RustEdition::Edition2024)
            .generate()
            .unwrap_or_else(|e| panic!("{target}: {e}"))
            .to_string()
    };
    let bindings = generate(TARGETS[0]);
    for target in &TARGETS[1..] {
        assert!(generate(target) == bindings, "bindings for {target} differ from {}", TARGETS[0]);
    }
    std::fs::write("src/bindings.rs", bindings).unwrap();
}
