#!/bin/sh
# Explicit opt-in: never materialise the wide source witness without a host cap.
set -eu
ulimit -v 8388608
ulimit -c 0
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
exec /usr/bin/time -f 'slow-source-forwarding wall_s=%e peak_rss_kib=%M' \
    timeout 600 cargo test --offline -j3 \
    --manifest-path "$root/reference-interpreter/Cargo.toml" \
    --test isa09_forwarding \
    source_forwarding_large_materialisation_reports_host_capacity_before_exhausting_memory \
    -- --ignored --exact --test-threads=1 --nocapture
