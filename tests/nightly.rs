use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Keep this file to one #[test]. Environment variables are process-wide, and
// libtest runs tests within the same test binary in parallel by default.

#[test]
fn find_nightly_installed_component_ignores_newer_stable() {
    let root = temp_rustup_home("nightly-ignores-newer-stable");
    let stable = add_toolchain(
        &root,
        "stable-x86_64-unknown-linux-gnu",
        "rustc 1.95.0 (000000000 2026-03-01)",
    );

    unsafe {
        std::env::set_var("RUSTUP_HOME", &root);
    }
    assert_eq!(
        toolchain_find::find_nightly_installed_component("rustfmt"),
        None
    );

    let nightly = add_toolchain(
        &root,
        "nightly-2026-01-01-x86_64-unknown-linux-gnu",
        "rustc 1.94.1-nightly (000000000 2026-01-01)",
    );
    assert_eq!(
        toolchain_find::find_installed_component("rustfmt"),
        Some(stable)
    );
    assert_eq!(
        toolchain_find::find_nightly_installed_component("rustfmt"),
        Some(nightly)
    );
    let _ = fs::remove_dir_all(root);
}

fn temp_rustup_home(test_name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("toolchain_find-{test_name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("toolchains")).unwrap();
    root
}

fn add_toolchain(root: &Path, toolchain: &str, rustc_version: &str) -> PathBuf {
    let bin = root.join("toolchains").join(toolchain).join("bin");
    fs::create_dir_all(&bin).unwrap();
    let rustfmt = bin.join(exe_name("rustfmt"));
    fs::write(&rustfmt, "").unwrap();
    let rustc = bin.join(exe_name("rustc"));
    write_fake_rustc(&rustc, rustc_version);
    rustfmt
}

fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn write_fake_rustc(path: &Path, version: &str) {
    let source = path.with_extension("rs");
    fs::write(
        &source,
        format!("fn main() {{ println!(\"{}\"); }}", version),
    )
    .unwrap();

    let status = Command::new("rustc")
        .env_remove("RUSTUP_HOME")
        .arg(&source)
        .arg("-o")
        .arg(path)
        .status()
        .unwrap();
    assert!(status.success());

    let _ = fs::remove_file(source);
}
