#!/usr/bin/env bash
# Symlink the stable toolchain's rust-analyzer-proc-macro-srv into the
# rust-mos sysroot so rust-analyzer can find a proc-macro server.
#
# Why: rust-mos 1.99 ships no rust-analyzer and no proc-macro server.
# .vscode/settings.json runs the stable toolchain's rust-analyzer with
# rust-analyzer.cargo.sysroot pointed at rust-mos; on startup the server
# probes <sysroot>/libexec/rust-analyzer-proc-macro-srv (then
# <sysroot>/lib/rust-analyzer-proc-macro-srv) and without the symlink
# fails with "cannot find proc-macro server in sysroot".
#
# The proc-macro .so files themselves are built by rust-analyzer's
# normal build-data pass (rustup shim -> mos-stable rustc), so they
# carry mos's rustc id. The stable proc-macro server logs a
# rustc-version-mismatch WARN when loading them; the WARN is not
# fatal (see CLAUDE.md).
#
# Re-run after any rust-mos reinstall (a fresh toolchain dir has an
# empty libexec). --unlink removes the symlink.

set -euo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"

STABLE_TOOLCHAIN="stable-x86_64-unknown-linux-gnu"
STABLE_SRV="$HOME/.rustup/toolchains/$STABLE_TOOLCHAIN/libexec/rust-analyzer-proc-macro-srv"

case "${1:-}" in
    "") ACTION=link ;;
    --unlink) ACTION=unlink ;;
    *) echo "usage: $0 [--unlink]" >&2; exit 2 ;;
esac

if [ ! -x "$STABLE_SRV" ]; then
    echo "error: stable proc-macro server not found at $STABLE_SRV" >&2
    echo "       run: rustup component add rust-analyzer --toolchain $STABLE_TOOLCHAIN" >&2
    exit 1
fi

# Resolve the active toolchain's sysroot (rust-mos) from the project dir.
MOS_SYSROOT="$(cd "$PROJECT_ROOT" && rustc --print sysroot)"
LINK="$MOS_SYSROOT/libexec/rust-analyzer-proc-macro-srv"

if [ "$ACTION" = unlink ]; then
    if [ -L "$LINK" ]; then
        rm -f "$LINK"
        echo "removed $LINK"
    else
        echo "no symlink to remove at $LINK"
    fi
    exit 0
fi

mkdir -p "$MOS_SYSROOT/libexec"
ln -sfn "$STABLE_SRV" "$LINK"
echo "linked $LINK -> $STABLE_SRV"
