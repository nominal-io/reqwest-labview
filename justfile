set working-directory := "rs-src"

default: ci

# Quick local check: lint, test
ci: clippy test

# Install the Rust targets the shared libraries are built for (matches CI)
setup:
    rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc x86_64-unknown-linux-gnu aarch64-apple-darwin

# Debug build of the shared library (also regenerates bindings.h)
build:
    cargo build

# Run tests
test:
    cargo test

# Run clippy lints
clippy:
    cargo clippy --all-targets

# Apply rust formatting
fmt:
    cargo fmt --all

# Check rust formatting
fmt-check:
    cargo fmt --all --check

# Release build for one target, as CI does; CI then renames it lib_reqwest_labview_<bits>.<ext>
package target="x86_64-pc-windows-msvc":
    cargo build --release --target {{target}}

# Copy lv-src/shared-library/lib_reqwest_labview_64.so to a LabVIEW RT target over ssh
deploy-rt ip:
    powershell -NoProfile -ExecutionPolicy Bypass -File ../lv-src/shared-library/deploy_rt_lib.ps1 -Ip {{ip}}
