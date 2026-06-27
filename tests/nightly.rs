use std::fs;

use test_helpers::{add_toolchain, temp_rustup_home};

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
