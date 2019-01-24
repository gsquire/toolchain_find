use toolchain_find::find_installed_component;

fn main() {
    println!("{:?}", find_installed_component("rustfmt"));
    println!("{:?}", find_installed_component("cargo-clippy"));
}
