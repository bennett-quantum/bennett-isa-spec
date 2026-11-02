#!/usr/bin/env bash
set -euo pipefail
unset DISPLAY WAYLAND_DISPLAY DBUS_SESSION_BUS_ADDRESS BROWSER
repo=/home/tobias/Projects/bennett-workspace/bennett-isa-spec
notes="$repo/docs/notes/2026-11-02-rev4"
export TMPDIR="$repo/.tmp/sol-rev4/tmp"
export CARGO_TARGET_DIR="$repo/.tmp/sol-rev4/target"
mkdir -p "$TMPDIR" "$CARGO_TARGET_DIR"
prlimit --as=6442450944 -- timeout 240 cargo build --offline --locked -j4 \
  --manifest-path "$repo/reference-interpreter/Cargo.toml"
prlimit --as=6442450944 -- timeout 60 rustc --edition=2021 \
  --extern "bennett_isa_ref=$CARGO_TARGET_DIR/debug/libbennett_isa_ref.rlib" \
  -L "dependency=$CARGO_TARGET_DIR/debug/deps" \
  "$notes/oracle_witness.rs" -o "$repo/.tmp/sol-rev4/oracle-witness"
prlimit --as=536870912 -- timeout 120 python3 -I -B "$notes/check_rev4.py"
prlimit --as=536870912 -- timeout 30 python3 -I -B \
  "$repo/docs/notes/2026-11-02-rev2/check_rev2.py"
