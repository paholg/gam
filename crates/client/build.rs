fn main() {
    let cr = i18n_config::Crate::from(env!("CARGO_MANIFEST_DIR"), None, "i18n.toml").unwrap();
    i18n_build::run(cr).unwrap();
}
