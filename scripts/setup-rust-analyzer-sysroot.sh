#!/usr/bin/env bash
# Pre-build the host-side proc-macro .so files using stable rustc so
# rust-analyzer's load on cold start accepts them.
#
# Why this exists: rust-mos 1.99 ships only the source tree and no
# prebuilt rust-analyzer; the host stable toolchain provides both. The
# .vscode/settings.json override routes proc-macro compilation through
# scripts/cargo-with-stable-rustc.sh so the host-target .so files are
# stamped with stable's rustc id, which is what stable's proc-macro
# server (bundled with the host rust-analyzer) accepts.
#
# Pre-warming means VS Code's first reload already finds matching .so
# files and skips the rustc-version-mismatch WARN. After toolchain
# swaps, re-run this script (or `cargo clean`) to refresh the host
# artifacts. Run with --unlink to do nothing (kept for backwards
# compat with the older sysroot-symlinking version of this script).
#
# Earlier versions also symlinked
# ~/.local/share/rust-mos/.../libexec/rust-analyzer-proc-macro-srv
# onto the stable toolchain's server. That was needed while
# `rust-analyzer.cargo.sysroot` pointed at the rust-mos sysroot. The
# current settings.json uses the stable sysroot, so rust-analyzer
# loads the proc-macro server from the stable toolchain directly and
# the symlink is no longer required.

set -euo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"

case "${1:-}" in
    --unlink|"") ;;
    *) echo "usage: $0 [--unlink]" >&2; exit 2 ;;
esac

if [ "${1:-}" = "--unlink" ]; then
    echo "(no symlinks to remove; sysroot-symlink mode is deprecated)"
    exit 0
fi

# Drop host-target proc-macro .so files that exist from any previous
# rust-mos build; rust-analyzer's proc-macro loader sees them on cold
# start and warns unless we either replace them with stable-built
# copies (next step) or wipe them.
pruned=0
shopt -s nullglob
for so in "$PROJECT_ROOT"/target/debug/deps/lib{ufmt_macros,thiserror_impl,defmt_macros,mos_test_macros}-*.so; do
    rm -f "$so"
    pruned=$((pruned + 1))
done
shopt -u nullglob

# Build host-target proc-macros with stable rustc so the .so files carry
# the stable rustc id, which is the one stable's proc-macro server
# accepts. Skip silently if cargo is unavailable; rust-analyzer will
# rebuild on its own the first time it starts.
prebuilt=0
if [ -x "$PROJECT_ROOT/scripts/cargo-with-stable-rustc.sh" ]; then
    if "$PROJECT_ROOT/scripts/cargo-with-stable-rustc.sh" build \
            -p ufmt-macros -p thiserror-impl -p defmt-macros -p mos-test-macros \
            --target x86_64-unknown-linux-gnu >/dev/null 2>&1; then
        prebuilt=1
    fi
fi

if [ "$pruned" -gt 0 ]; then
    echo "pruned $pruned stale host-target proc-macro .so artifact(s) under target/debug/deps/"
fi
if [ "$prebuilt" -eq 1 ]; then
    echo "pre-built host-side proc-macros with stable rustc"
fi
