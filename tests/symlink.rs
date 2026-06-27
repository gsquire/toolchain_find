use std::path::Path;
use std::process::Command;
use std::{env, fs};

use googletest::prelude::*;
use test_helpers::{add_toolchain, temp_rustup_home};
use toolchain_find::find_installed_component;

#[test]
fn find_installed_component_follows_symbolic_links() {
    let sym_home = temp_rustup_home("symlink");
    let _ = add_toolchain(
        &sym_home,
        "stable-x86_64-unknown-linux-gnu",
        "rustc 1.95.0 (000000000 2026-03-01)",
    );

    unsafe {
        env::set_var("RUSTUP_HOME", &sym_home);
    }

    let target = Path::new(&sym_home)
        .join("toolchains")
        .join("stable-x86_64-unknown-linux-gnu");
    let status = Command::new("rustup")
        .arg("toolchain")
        .arg("link")
        .arg("zzz")
        .arg(target)
        .status()
        .unwrap();
    assert!(status.success());

    // Sort of a hack but we're relying on the fact that our components are sorted before being
    // returned.
    assert_that!(
        find_installed_component("rustfmt")
            .unwrap()
            .to_str()
            .unwrap(),
        contains_substring("zzz")
    );

    let _ = fs::remove_dir_all(sym_home);
}
