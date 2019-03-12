use toolchain_find;

#[test]
fn cross_platform() {
    let path = toolchain_find::find_installed_component("rustfmt");
    assert!(path.is_some());
}
