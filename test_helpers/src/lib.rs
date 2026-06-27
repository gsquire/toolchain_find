use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn temp_rustup_home(test_name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("toolchain_find-{test_name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("toolchains")).unwrap();
    root
}

pub fn add_toolchain(root: &Path, toolchain: &str, rustc_version: &str) -> PathBuf {
    let bin = root.join("toolchains").join(toolchain).join("bin");
    fs::create_dir_all(&bin).unwrap();
    let lib = root.join("toolchains").join(toolchain).join("lib");
    fs::create_dir_all(&lib).unwrap();

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
