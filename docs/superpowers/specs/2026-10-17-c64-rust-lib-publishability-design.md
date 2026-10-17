# c64-rust-lib publishability floor

**Date:** 2026-10-17
**Status:** Draft — pending review
**Scope:** metadata, docs, lint enforcement, unsafe auditing, CI, virtual-manifest cleanup. No new public API. No error-type migration (deferred). No new feature work.

## Goal

Lift `c64-rust-lib` from "working playground" to "publishable C64 Rust library" by closing the gaps that matter for someone discovering the crates on docs.rs or `cargo search` — without expanding the API surface or touching unrelated code.

Out of scope: new features (sprite/multicolor helpers, raster IRQ wrapper, SID playback, `REUArray` ergonomics), error-type migration, REU trait abstraction, CX16/MEGA65 compatibility layer, high-level engine abstractions. These are tracked as future specs.

## Context

- Workspace at `/home/qus/dev/_c64/c64-rust-lib` with members `reu`, `plotek`, `ultimate64`, `demo`.
- Target: `mos-c64-none` via `mos-stable` Rust toolchain (`rustc 1.99.1 mos`). Default toolchain pin is `v1.99.0-mos.2` per `CLAUDE.md`.
- `vendor/mos-hardware` is a local fork of upstream commit `425eb7df` patched for Rust 1.99, C64 re-export, and a SID loader change.
- Existing tests: `reu/tests/test.rs` uses `mos-test` + `defmt-test` for register-layout and bitflag checks that run without hardware. `demo/src/reu_tests.rs` and `demo/src/plotek_tests.rs` exercise hardware paths.
- Recent commits are bug fixes and formatting churn; no CI, no per-crate metadata, no `#![deny(missing_docs)]`, no MSRV pin.

## Design

### 1. Workspace root becomes a virtual manifest

The root `Cargo.toml` carries a `[package]` named `c64-rust-lib`, but the actual library code lives in members. `cargo publish` from the root would publish an empty library. Fix:

- Move the root `[package]` and root `[dependencies]` into `demo/Cargo.toml` (demo already depends on every member).
- Replace root `Cargo.toml` with a virtual manifest: `[workspace]` only, with `resolver = "2"`.
- Delete the root `build.rs` — it currently builds `reu/src/fake_interrupt.S` redundantly; `reu/build.rs` already handles it for `mos-c64` targets.
- Delete `vendor/mos-hardware` from the root `[patch.crates-io]` and put it in each consuming member's manifest, or leave it at workspace level with `patch` syntax adjusted for virtual manifests. **Decision:** keep `[patch.crates-io]` at workspace level — virtual manifests support `[patch]`.
- Keep `CLAUDE.md` at root — it documents the workspace, not a crate.

Result: `cargo build` and `cargo build -p demo --features reu-hardware-tests` still work; `cargo publish -p reu` becomes meaningful.

### 2. Per-crate metadata

For each of `reu`, `plotek`, `ultimate64`, `demo` add to `Cargo.toml`:

```toml
description = "<one-line purpose>"
license = "MIT OR Apache-2.0"
repository = "https://github.com/<owner>/c64-rust-lib"
documentation = "https://docs.rs/<crate>"
readme = "README.md"            # only if a README exists at that crate root
keywords = ["c64", "mos-hardware", "no-std", "embedded"]
categories = ["embedded", "no-std::no-alloc"]
```

Add a per-crate `README.md` where one doesn't exist, summarising what the crate does, the target audience, and a minimal usage example. The `plotek` README can lean on the existing top-level README's hires example.

`description` text (proposed):

| Crate | Description |
|---|---|
| `reu` | Commodore 64 / 128 RAM Expansion Unit (REU) driver: DMA wrappers, 256-byte-unit allocator, and a windowed array backed by REU memory. |
| `plotek` | C64 hires and text-mode screen primitives with VIC bank selection and bitmap-mode switching. |
| `ultimate64` | Ultimate 64 (`$D030`/`$D031`) turbo-register accessors. |
| `demo` | Demo binary exercising `reu`, `plotek`, and `ultimate64` on `mos-c64-none`. |

### 3. Crate-level docs lint

Add to each library crate's `lib.rs`:

```rust
#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(missing_docs)]  // fallback if deny breaks existing code
```

Implementation order: add `warn` first, fix violations, then upgrade to `deny`. The audit must produce zero warnings before `deny` lands.

Add a crate-level doc comment (`//!`) to each library crate summarising purpose, hardware target, and the minimum required hardware (REU size for `reu`, VIC bank assumptions for `plotek`, Ultimate 64 board for `ultimate64`).

`plotek` and `ultimate64` already have crate-level docs in spirit (file headers). `reu/src/lib.rs` is bare — needs a proper `//!` block.

### 4. `Debug` implementations

- `ReuChunk`: add `#[derive(Clone, Copy, Debug)]`. Layout is two `u32` fields; no PackedStruct concerns.
- `RamExpanstionUnit`: cannot derive `Debug` safely (packed repr, volatile pointers). Add a manual `impl core::fmt::Debug for RamExpanstionUnit` that prints a static name, or skip — `Debug` on a hardware-register type is rarely useful. **Decision:** skip; document the omission in the spec.
- `REUArray<T>`: add a manual `Debug` that prints `REUArray { len, capacity, element_size }` without dereferencing the cache pointer (the cache contents live in RAM; Debug on them would require unsafe).

### 5. Unsafe audit and `// SAFETY:` comments

Walk every `unsafe { ... }` block in `reu/`, `plotek/`, `ultimate64/` and add a one-line `// SAFETY: ...` rationale immediately above. Inventory of unsafe sites (initial pass):

| File | Location (rough) | Existing rationale | Required rationale |
|---|---|---|---|
| `reu/src/ram_expansion_unit.rs` | `REU` static, `set_range`, `pull`/`push`/`swap`/`fill`/`fill_reu` | none | Volatile MMIO writes to `$DF00`; caller must respect C64 address and length invariants already enforced by `set_range`. |
| `reu/src/reu_allocator.rs` | `BOM` static, `swap_bom_in/out`, `dealloc`, `CPU_PORT` writes | none | Static at `$E000` is reserved by the allocator; CPU port flip is gated by `__disable_mi`/`__enable_mi` and restored in reverse order. |
| `reu/src/vectors.rs` | `INTERRUPT_VECTORS` static | none | Read-only access to the fixed `$FFFA` vector area. |
| `reu/src/reu_array.rs` | `malloc`/`free` extern, raw pointer derefs, `cache` reads/writes | none | `malloc`/`free` resolve to `mos-alloc`; cache pointer is heap-allocated and indexed with `check_bounds`. |
| `plotek/src/lib.rs` | `CLEAR_FUNC` static, `set_bitmap_mode`, `show`/`hide`, `buffer_ptr` | none | VIC register writes at `$D011`/`$D018`; `CLEAR_FUNC` is initialised before `show`/`hide`. |
| `plotek/src/cia2.rs` | `CIA2` static, `set_vic_bank` | none | CIA2 register reads/writes at `$DD00`/`$DD02`. |
| `ultimate64/src/lib.rs` | `ULTIMATE` static, `set_speed`/`set_enable` | none | U64 turbo-register writes at `$D030`/`$D031`. |

Convention: every `unsafe fn`, `unsafe { }` block, and `unsafe trait`/`impl` gets a `// SAFETY:` comment. Use `cargo geiger` (added in step 7 CI) to verify coverage.

### 6. Examples for docs.rs

Add `examples/` to each library crate. One example per crate is enough.

- `reu/examples/array_basic.rs` — `with_capacity`, `push`, `get`, `for_each_mut`, drop. Comments explain the allocator setup (`init_allocator(8 * 1024 * 1024)`).
- `plotek/examples/hires_lines.rs` — the `test_hires`-style demo from the top-level README, trimmed to a single `main`.
- `ultimate64/examples/turbo.rs` — set 48 MHz, enable turbo, halt with `cbm_k_chrin` so a human can see the speed change.

Examples must build with `cargo build --target mos-c64-none --example <name>` and run in VICE. They share the `demo`'s panic-handler pattern (`#[no_std]` + `#[no_main]` + `extern "C" fn main`).

### 7. CI

`.github/workflows/ci.yml`:

```yaml
name: ci
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/sbechet/rust-mos:beta
    steps:
      - uses: actions/checkout@v4
      - name: Build workspace
        run: cargo build --workspace --target mos-c64-none
      - name: Build with hardware tests
        run: cargo build -p demo --target mos-c64-none --features reu-hardware-tests
      - name: Doc
        run: cargo doc --workspace --no-deps --target mos-c64-none
      - name: Clippy
        run: cargo clippy --workspace --target mos-c64-none -- -D warnings
      - name: Geiger (unsafe inventory)
        run: cargo install --locked cargo-geiger && cargo geiger --workspace
```

The container image matches `.devcontainer/devcontainer.json`. No secrets needed. No caching for v1 — `rust-mos` image is large enough that `rustup`/`cargo` setup is the slow part and the image already has it.

`cargo geiger` produces a markdown table of unsafe usage per crate; it doesn't fail the build, but the report is committed as a CI artifact for review. Long-term, set a per-crate ceiling.

### 8. MSRV and toolchain pin

Add `rust-version = "1.99"` to each library crate's `Cargo.toml`. Stay on `mos-stable` channel via `rust-toolchain.toml`; document that the MSRV is the bundled `v1.99.0-mos.2` in `CLAUDE.md`.

## Acceptance criteria

1. Root `Cargo.toml` is a virtual manifest; `cargo build` from root builds all members; `cargo publish --dry-run -p reu` succeeds.
2. Each library crate has `description`, `license`, `repository`, `keywords`, `categories`, and a `README.md`. `cargo publish --dry-run` per crate is clean.
3. Each library crate has a `//!` crate-level doc and `#![deny(missing_docs)]` with zero warnings.
4. Every `unsafe` block in `reu/`, `plotek/`, `ultimate64/` carries a `// SAFETY:` comment above it. `cargo geiger` output is reviewed and committed.
5. Each library crate has at least one `examples/` file that builds with `cargo build --example <name> --target mos-c64-none`.
6. `.github/workflows/ci.yml` runs build + doc + clippy + geiger on push and PR; the green build is the signal.
7. `CLAUDE.md` is updated to reflect the virtual manifest and the new MSRV pin.
8. `cargo build -p demo --target mos-c64-none` and `cargo build -p demo --target mos-c64-none --features reu-hardware-tests` both succeed.

## Risk and rollback

- **Risk:** `#![deny(missing_docs)]` reveals that the `unsafe` externs `__enable_mi` / `__disable_mi` in `reu/src/lib.rs` lack docs. Mitigation: add docs or use `#[allow(missing_docs)]` only on those two lines.
- **Risk:** Virtual-manifest move breaks someone running `cargo build` from the root with extra flags. Mitigation: keep all member `[package]` names unchanged; root has no `[package]` so workspace commands work identically.
- **Risk:** Adding `cargo geiger` to CI introduces a heavy install step. Mitigation: pin to a specific version (`cargo install --locked`); cache `$CARGO_HOME` across runs in a follow-up.
- **Rollback:** each change is independently revertible. CI failure rolls back via GitHub UI; metadata changes are doc-only.

## Out-of-scope follow-ups (separate specs later)

- `ReuError` enum + fallible variants (breaking 0.2.0).
- `plotek` sprite/multicolor API.
- `RasterIrq::install` wrapper.
- SID playback helper.
- `REUArray` indexing (`Index<u32>`), `Default`, in-place iterators.
- `ReuBus` trait + `mock` feature.