fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/cxx.rs");

    cxx_build::bridge("src/cxx.rs")
        .flag_if_supported("-std=c++17")
        .compile("arkcxxbridge");
}
