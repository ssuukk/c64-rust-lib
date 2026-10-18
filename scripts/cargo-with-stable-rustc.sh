#!/usr/bin/env bash
# Build-script / proc-macro cargo wrapper for rust-analyzer.
#
# rust-analyzer hard-codes its build-script invocation as:
#   cargo check --quiet --workspace --message-format=json --all-targets --keep-going
# and uses the active rustc, which on this workspace is rust-mos
# (channel = "mos-stable"). rust-mos-built proc-macro dylibs carry the
# rust-mos build-id stamp, and the rust-analyzer proc-macro server bundled
# with the "stable" toolchain rejects them with
#   "rustc version mismatch. expected ... (58cf8f9c3), found ... (mos)".
#
# This wrapper:
#   1. drops any --target flag rust-analyzer forwarded (cargo's default
#      target is mos-c64-none here, which stable rustc rejects with
#      "could not find specification for target mos-c64-none");
#   2. injects --target x86_64-unknown-linux-gnu so host proc-macros get
#      compiled by stable rustc, producing .so files stamped with the
#      stable rustc id that rust-analyzer's proc-macro server accepts;
#   3. runs stable cargo from the matching toolchain.
#
# Library / binary builds (cargo build, cargo build --target mos-c64-none)
# do not go through this wrapper and remain on mos-stable.

set -euo pipefail

ARGS=()
had_target=0
consume_next=0
for arg in "$@"; do
    if [ "$consume_next" -eq 1 ]; then
        consume_next=0
        continue
    fi
    case "$arg" in
        --target)
            had_target=1
            consume_next=1
            ;;
        --target=*)
            had_target=1
            ;;
        *)
            ARGS+=("$arg")
            ;;
    esac
done

if [ "$had_target" -eq 0 ]; then
    ARGS=(--target x86_64-unknown-linux-gnu "${ARGS[@]}")
fi

exec env -i \
    HOME="$HOME" \
    PATH="/home/qus/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:$PATH" \
    RUSTUP_HOME="${RUSTUP_HOME:-${HOME}/.rustup}" \
    CARGO_HOME="${CARGO_HOME:-${HOME}/.cargo}" \
    /home/qus/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/cargo "${ARGS[@]}"
