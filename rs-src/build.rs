fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = std::path::PathBuf::from(&crate_dir).join("bindings.h");

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("HTTP_RS_LABVIEW_H")
        .with_std_types(true)       // Emits uint32_t, int32_t etc. from <stdint.h>
        .with_documentation(true)   // Includes doc comments as C comments
        .with_sys_include("stdint.h")
        .generate()
        .expect("Unable to generate C bindings")
        .write_to_file(output_file);
}