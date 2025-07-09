#[cxx::bridge(namespace = "bark_cxx")]
mod ffi {
    extern "Rust" {
        fn create_mnemonic() -> Result<String>;
    }
}

fn create_mnemonic() -> anyhow::Result<String> {
    crate::create_mnemonic()
}
