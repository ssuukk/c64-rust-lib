# Build Notes

- Use the Rust MOS 1.99.0 toolchain from `v1.99.0-mos.2`. Check `rustc --version`; the container's default compiler may be a different MOS beta.
- The default target is `mos-c64-none`. Run `cargo build --workspace`; do not enable `build-std`, since the toolchain provides prebuilt `core` and `alloc`.
- Keep LTO disabled unless the toolchain libraries support embedded bitcode. Enabling it caused rustc to panic during linking.
- `vendor/mos-hardware` is a local compatibility copy based on upstream commit `425eb7df`. Preserve its Rust 1.99 compatibility changes, C64 API re-export, and SID loader change when updating it.
- The build artifact is `target/mos-c64-none/debug/demo`. VICE (`x64sc` or `x64`) was not installed in the container, so emulator execution has not been verified.