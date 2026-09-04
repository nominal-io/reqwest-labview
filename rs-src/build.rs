fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = std::path::PathBuf::from(&crate_dir).join("bindings.h");

    cbindgen::Builder::new()
        .with_crate(crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("REQWEST_LABVIEW_H")
        .with_std_types(true)       // Emits uint32_t, int32_t etc. from <stdint.h>
        .with_documentation(true)   // Includes doc comments as C comments
        .with_sys_include("stdint.h")
        .generate()
        .expect("Unable to generate C bindings")
        .write_to_file(output_file);

    embed_version_metadata();
}

/// Embed version metadata in the produced library.
///
/// Windows: a VERSIONINFO resource (CompanyName, FileDescription, versions).
/// An unsigned DLL with empty version metadata is a feature AV heuristics
/// weight against us, so fill it in.
///
/// macOS: Mach-O carries current/compatibility versions in LC_ID_DYLIB,
/// set through linker flags.
///
/// Linux: ELF has no embedded version metadata; versioning is by filename
/// convention only. Callers on any OS can query http_get_version() instead.
fn embed_version_metadata() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "windows" {
        embed_windows_resource();
    }

    if target_os == "macos" {
        let version = std::env::var("CARGO_PKG_VERSION").unwrap();
        println!("cargo:rustc-link-arg=-Wl,-current_version,{version}");
        println!("cargo:rustc-link-arg=-Wl,-compatibility_version,{version}");
    }
}

#[cfg(windows)]
fn embed_windows_resource() {
    let mut res = winresource::WindowsResource::new();
    // FileVersion/ProductVersion are prefilled from CARGO_PKG_VERSION.
    res.set("CompanyName", "nominal-io")
        .set("FileDescription", "reqwest HTTP client for LabVIEW")
        .set("ProductName", "reqwest-labview")
        .set("LegalCopyright", "Copyright (c) 2026 nominal-io, MIT License")
        .set("OriginalFilename", "reqwest_labview.dll");
    res.compile()
        .expect("Failed to compile Windows version resource");
}

// The Windows targets are only built on Windows hosts (see build.yml), where
// the cfg(windows) build-dependency and function above take effect.
#[cfg(not(windows))]
fn embed_windows_resource() {}
