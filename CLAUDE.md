# Build Notes

- Use the Rust MOS 1.99.0 toolchain from `v1.99.0-mos.2`. Check `rustc --version`; the container's default compiler may be a different MOS beta.
- The default target is `mos-c64-none`. Run `cargo build --workspace`; do not enable `build-std`, since the toolchain provides prebuilt `core` and `alloc`.
- Keep LTO disabled unless the toolchain libraries support embedded bitcode. Enabling it caused rustc to panic during linking.
- `vendor/mos-hardware` is a local compatibility copy based on upstream commit `425eb7df`. Preserve its Rust 1.99 compatibility changes, C64 API re-export, and SID loader change when updating it.
- The build artifact is `target/mos-c64-none/debug/demo`. VICE (`x64sc`) testing recipe below.

# VICE Testing

`x64sc` is available on this system. Emulator runs are flaky on small Xvfb displays; intermittent `malloc(): unaligned tcache chunk detected` crashes are an Xvfb/VICE GL issue, not the test. Persist through retries.

## Working invocation

```sh
# 1. Start a writable HOME (VICE writes ~/.cache/vice)
mkdir -p /tmp/vice-home/.cache

# 2. Use a 1024x768 Xvfb on :99 (640x480 segfaults VICE on init)
Xvfb :99 -screen 0 1024x768x24 -nolisten tcp &
sleep 1

# 3. Build the demo (REU tests need the feature flag)
cargo build -p demo --target mos-c64-none --features reu-hardware-tests
cp target/mos-c64-none/debug/demo /tmp/demo.prg

# 4. Run VICE with 8 MiB REU + stock C64 ROMs (NEVER OpenROMs)
DISPLAY=:99 HOME=/tmp/vice-home x64sc \
  -reu -reusize 8192 -remotemonitor \
  -autostart /tmp/demo.prg \
  -kernal   /home/qus/.local/share/vice/C64/kernal-901227-02.bin \
  -basic    /home/qus/.local/share/vice/C64/basic-901226-01.bin \
  -chargen  /home/qus/.local/share/vice/C64/chargen-901225-01.bin \
  -logfile  /tmp/vice.log
# wait ~8s for port 6510
sleep 8; ss -tlnp | grep x64
```

## Reading state

Remote monitor accepts one query per TCP connection. One query per `ncat`:

```sh
{ printf 'm d020\r\n'; sleep 0.4; } | timeout 4 ncat 127.0.0.1 6510
{ printf 'm 0400 0428\r\n'; sleep 0.4; } | timeout 4 ncat 127.0.0.1 6510
{ printf 'r PC\r\n'; sleep 0.4; } | timeout 4 ncat 127.0.0.1 6510
```

`m <hexaddr>` returns 16 bytes from that address. The first byte after the `>C:XXXX` prompt is the value at the requested address. Example: `m d020` returns `fe f6 f1 f2 …` — the `fe` is the border color; low nibble `0x0E` is the C64 light-blue default.

## Pass/fail signal

- **Success:** VICE log contains `AUTOSTART: Loading PRG file '/tmp/demo.prg'` and `AUTOSTART: Starting program.`, then continues with `VSync: Sync reset` (emulator is alive, not in panic `loop {}`).
- **No panic:** `grep -i panic /tmp/vice*.log` returns nothing. The C64 panic handler does `loop {}`, which freezes the emulator — no further `VSync` entries would appear.
- **Screen content is not a panic signal.** Decoded PETSCII is hard to read; the C64 chargen at `$5E` is the up-arrow glyph, and the screen may fill with it for unrelated reasons. Trust the VICE log, not the screen.

## Known gotchas

- **Display :99 not :101:** 640x480 Xvfb on `:101` causes VICE to segfault on init in `__fopen_internal`. Use 1024x768 on `:99`.
- **Writable HOME:** Without `HOME=/tmp/vice-home` (or similar), VICE prints `Error - failed to create user cache dir` and dies.
- **Don't poll `pgrep`/`ss` in a tight loop** — VICE startup is slow, give it 8-10 s.
- **VICE 3.10 occasionally crashes with `malloc(): unaligned tcache chunk detected` on display change.** Just retry. If multiple retries fail, reboot the Xvfb session.
- **Remote monitor is one-shot per connection** — issuing a second `m` over the same TCP connection hangs. Use fresh `ncat` invocations.
- **The `vice_network_send: Przerwany potok (broken pipe)` in the log is the sound buffer overrun**, not a C64 program error. Ignore.
- **`Sound: Buffer full during suspend` in the log is the VICE sound subsystem**, not our code. Ignore.

# Editor setup (VS Code + rust-analyzer)

rust-mos 1.99 sysroot does NOT ship its own `rust-analyzer` or `rust-analyzer-proc-macro-srv`. Without help, the editor aborts expansion with `cannot find proc-macro server in sysroot /home/qus/.local/share/rust-mos/v1.99.1-mos.1-...`, which cascades into false positives like `cannot index into a value of type REUArray<T>` (`Index<u32>` already exists in `reu/src/reu_array.rs`). One-time setup, re-run after any rust-mos reinstall:

```sh
rustup component add rust-analyzer --toolchain stable-x86_64-unknown-linux-gnu
./scripts/setup-rust-analyzer-sysroot.sh    # symlink rust-analyzer-proc-macro-srv into the rust-mos sysroot
```

To undo the sysroot patch:

```sh
./scripts/setup-rust-analyzer-sysroot.sh --unlink
```

`.vscode/settings.json` points rust-analyzer at the stable binary, sets `rust-analyzer.cargo.target = mos-c64-none`, leaves `rust-analyzer.rustc.source = null`, and disables on-save checks (`rust-analyzer.checkOnSave = false`). The project toolchain pin stays `mos-stable`; cargo builds are unaffected.

Gotchas fixed by recent schema adjustments:

- **`rust-analyzer.checkOnSave` is a plain boolean.** The old `rust-analyzer.checkOnSave.allTargets` is invalid; use the top-level `rust-analyzer.check.allTargets` instead.
- **`rust-analyzer.rustc.source`** is the camelCase form in current builds; setting the lowercase `rustcSource` is rejected.

Known second-order noise NOT fixed by the symlink: rust-analyzer pre-builds host-target `target/debug/deps/lib{ufmt_macros,thiserror_impl,defmt_macros,mos_test_macros}-*.so`, while the active rustc is mos-stable. The hashes differ; logs show `proc macro server error: rustc version mismatch. expected rustc 1.99.1 (58cf8f9... ), found rustc 1.99.1 (6f73c1e3b... ) (mos)`. These are WARNs, not fatal. `cargo clean` or `cargo clean -p ufmt-macros -p thiserror-impl -p defmt-macros -p mos-test-macros` suppresses them after switching rustc versions.
- **Verify:** After running the script once, restart VS Code (or run `> Rust Analyzer: Restart Server`) and confirm the diagnostics on `demo/src/reu_tests.rs` no longer show the two original errors. The remaining rust-analyzer log noise (`can't load standard library`, host-target proc-macro `.so` version-mismatch WARNs) is residual and unrelated.

Background noise and the configurations that suppress it (NOT recommended unless the noise blocks your work):

- **`ERROR can't load standard library, sysroot_path=.../rust-mos/...`** — rust-mos ships `core`/`alloc` as prebuilt rlibs without `rust-src`. rust-analyzer asks for the source so go-to-def jumps into `core`. Mitigated by `rust-analyzer.cargo.sysrootSrc = null` (already set); click-throughs to `core::` items land on the rlib symbol instead. To silence the log entirely, set `rust-analyzer.cargo.noDeps = true` (loses go-to-def into dependencies).
- **`WARN proc-macro ... rustc version mismatch`** — rust-analyzer's build-script loader rebuilds host-target proc-macro `.so` files on every cold start; they no longer match the mos-stable rustc hash, but they aren't actually used for `mos-c64-none` analysis. The loader logs a WARN per `.so`. To silence, set `rust-analyzer.procMacro.enable = false` (loses derive expansion in editor diagnostics).

