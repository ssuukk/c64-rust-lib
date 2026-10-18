#!/usr/bin/env bash
# rust-analyzer build-data builder (rust-analyzer.cargo.buildScripts.overrideCommand).
#
# Builds the compile-time deps (proc-macro .so) with the host "stable"
# toolchain into an isolated target dir. Rationale:
#
# - rust-analyzer's proc-macro server is the host stable binary; it only
#   loads .so stamped with the stable rustc version. .so built by the
#   mos-stable workspace build are rejected:
#     "rustc version mismatch. expected rustc ... (stable), found ... (mos)"
# - A stable build in the shared target/ dir fails with E0514 (mixed
#   mos/stable rustc metadata), so the stable build uses target/ra-stable.
#
# The C64 workspace members' build scripts (mos-c64-clang) cannot run for
# the host target; their failures are expected. The script exits 0 as long
# as all four proc-macro .so exist; cargo stderr is kept in
# target/ra-stable/ra-build.log.

set -u
cd "$(dirname "$0")/.."

STABLE_BIN="/home/qus/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin"
TARGET_DIR="target/ra-stable"
LOG="$TARGET_DIR/ra-build.log"
mkdir -p "$TARGET_DIR"

# JSON on stdout is streamed verbatim to rust-analyzer.
env __CARGO_TEST_CHANNEL_OVERRIDE_DO_NOT_USE_THIS=nightly \
  "$STABLE_BIN/cargo" check \
    --quiet --workspace --message-format=json \
    --target x86_64-unknown-linux-gnu --target-dir "$TARGET_DIR" \
    --all-features --all-targets --keep-going \
    --compile-time-deps -Zunstable-options \
    2>"$LOG"

ok=1
for so in libufmt_macros libthiserror_impl libdefmt_macros libmos_test_macros; do
  ls "$TARGET_DIR"/debug/deps/"$so"-*.so >/dev/null 2>&1 || ok=0
done
if [ "$ok" -ne 1 ]; then
  echo "ra-build-data: proc-macro .so missing; see $LOG" >&2
  exit 1
fi
exit 0
