use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn command_output(program: &str, args: &[&str]) -> String {
    let output = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {program}: {e}"));
    if !output.status.success() {
        panic!(
            "{program} {} failed:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8(output.stdout)
        .expect("tool output must be UTF-8")
        .trim()
        .to_string()
}

fn compile_macos_app_intents(manifest_dir: &Path, target: &str, out_dir: &Path) {
    let arch = if target.starts_with("aarch64-") {
        "arm64"
    } else if target.starts_with("x86_64-") {
        "x86_64"
    } else {
        panic!("unsupported macOS architecture for App Intents: {target}");
    };
    let deployment = env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "13.0".into());
    let swift_target = format!("{arch}-apple-macosx{deployment}");
    let source = manifest_dir.join("gen/apple/Sources/hp41-gui/HP41AppIntents.swift");
    let object = out_dir.join("HP41AppIntents.o");

    println!("cargo:rerun-if-changed={}", source.display());
    println!("cargo:rerun-if-env-changed=DEVELOPER_DIR");
    println!("cargo:rerun-if-env-changed=MACOSX_DEPLOYMENT_TARGET");

    let status = Command::new("xcrun")
        .args(["--sdk", "macosx", "swiftc"])
        .arg(&source)
        .args([
            "-parse-as-library",
            "-target",
            &swift_target,
            "-module-name",
            "HP41AppIntents",
            "-emit-object",
            "-o",
        ])
        .arg(&object)
        .status()
        .expect("failed to launch the Swift compiler for macOS App Intents");
    assert!(
        status.success(),
        "Swift compilation for macOS App Intents failed"
    );

    // Link the target-specific Swift object directly into the Tauri executable.
    // The object contains a C callback reference implemented in Rust, which acts
    // as a link anchor and prevents the App Intent conformances being discarded.
    println!("cargo:rustc-link-arg-bin=hp41-gui={}", object.display());
    println!("cargo:rustc-link-lib=framework=AppIntents");
    println!("cargo:rustc-link-lib=framework=Foundation");
    println!("cargo:rustc-link-arg-bin=hp41-gui=-Wl,-rpath,@executable_path");
    println!("cargo:rustc-link-arg-bin=hp41-gui=-Wl,-rpath,/usr/lib/swift");
    println!("cargo:rustc-link-arg-bin=hp41-gui=-Wl,-rpath,@executable_path/../Frameworks");

    let sdk = command_output("xcrun", &["--sdk", "macosx", "--show-sdk-path"]);
    println!("cargo:rustc-link-search=native={sdk}/usr/lib/swift");
    let swiftc = PathBuf::from(command_output("xcrun", &["--find", "swiftc"]));
    if let Some(toolchain_usr) = swiftc.parent().and_then(Path::parent) {
        let compatibility_dir = toolchain_usr.join("lib/swift-5.5/macosx");
        println!(
            "cargo:rustc-link-search=native={}",
            toolchain_usr.join("lib/swift/macosx").display()
        );
        println!(
            "cargo:rustc-link-search=native={}",
            compatibility_dir.display()
        );
    }
}

fn main() {
    let target = env::var("TARGET").expect("Cargo must set TARGET");
    if target.ends_with("-apple-darwin") {
        let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        compile_macos_app_intents(&manifest_dir, &target, &out_dir);
    }

    tauri_build::build()
}
