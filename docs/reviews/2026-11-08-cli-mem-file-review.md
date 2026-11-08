# Reference CLI `--mem-file` review

Reviewer: **gpt-6.1-sol** (fresh review, round 1). Date (UTC): **2026-11-08T11:25:17Z**.
Reviewed commit: **c3714ca037d69699ced9bea671ccf899d30d1719**; base: **62a0cf0ef9f1f1c1db12c84ed24846b6b9695cdc**.
Scope: `git diff 62a0cf0..c3714ca` (four files). Read the brief, COMMON.md, T3CHILD.md, repository instructions, implementer progress, red/green logs and receipts, and `bd show b-isa-spec-mtc` from the main checkout. No code or Beads changes made.

## Findings

1. **P2, proposed bead: name unknown region assignments in oracle CLI diagnostics.** This is the sole diagnostic nit and reproduces on the base. After the shared parser accepts `unknown_region`, `run-oracle`, `step-check` and oracle `reverse` add the declared region's zero entry and reject with only `§5.1: interface region count mismatch`. The diagnostic omits the supplied name; action and classical paths already identify it. For example, with `BIN=.tmp/isa-mtc-review/target/debug/bennett-isa-ref`:

   ```sh
   "$BIN" run-oracle reference-interpreter/tests/fixtures/tiny_memory.bisa tiny --input 1 --budget 20 --mem-file "unknown_region=$PWD/.tmp/isa-mtc-review/edges/0 raw=data.bin"
   ```

   Result: exit 1, empty stdout, stderr `§5.1: interface region count mismatch`. Substituting `--mem unknown_region=a5` reproduces the same diagnostic on both the candidate and the separately compiled `62a0cf0` binary. Locations: `reference-interpreter/src/main.rs:1640` and `:2192` (accept assignment), then `reference-interpreter/src/oracle.rs:207` (generic count check). Proposed work: validate supplied names against the selected oracle's mem parameters before filling omitted regions, and name the unknown assignment for both flags, with regressions for all three commands. No silent fallback, changed interpreter result, or broken build was observed; this is P2 under rule 7. No Beads write was authorized, so the orchestrator must file this proposal.

No P0/P1 finding. No additional nit.

## Independent re-runs

All commands ran in `/home/tobias/Projects/bennett-workspace/bennett-isa-spec-job-isa-mtc`, with DISPLAY, WAYLAND_DISPLAY, DBUS_SESSION_BUS_ADDRESS and BROWSER unset, `TMPDIR=$PWD/.tmp/isa-mtc-review/tmp`, and `CARGO_TARGET_DIR=$PWD/.tmp/isa-mtc-review/target`. The focused runner reads the existing Cargo guard from `gate.toml`, records wall/CPU receipts, and maps a guard trip to UNKNOWN/124. The repository has no machine profile/class table. Width 4 was derived from nproc=64 and approximately 53 GiB available memory, leaving headroom; the standing 12-core/30 GB process-rule precondition does not match this host. No full gate or sibling gate was run.

- **Red:** archive `62a0cf0` under `.tmp/isa-mtc-review/baseline`, then overlay only the candidate's new test and two fixtures. Run `python3 -I .tmp/isa-mtc-review/run-focused.py red cargo test --offline --locked -j4 --manifest-path .tmp/isa-mtc-review/baseline/reference-interpreter/Cargo.toml --test cli_mem_file -- --test-threads=4`. Exit **101**, **0 passed / 6 failed**, all rejecting unknown `--mem-file`; **36.849876 wall / 121.065675 CPU seconds**. This independently reproduces the author's red-first claim.
- **Green:** `python3 -I .tmp/isa-mtc-review/run-focused.py green cargo test --offline --locked -j4 --manifest-path reference-interpreter/Cargo.toml --test cli --test cli_xam --test cli_mem_file -- --test-threads=4`. Exit **0**, **92 passed** (6 + 80 + 6), zero failures or ignored tests; **74.409773 wall / 164.113688 CPU seconds**.
- **Independent edges and witness:** `python3 -I .tmp/isa-mtc-review/run-focused.py edges python3 -I .tmp/isa-mtc-review/check-edges.py`. Exit **0**, **162 asserted CLI invocations**, **0.515825 wall / 0.514370 CPU seconds**. Eight dispatch cases cover all six subcommands and all three reverse kinds. Verified raw NUL/newline/non-UTF-8 bytes, paths containing spaces and `=`, byte-identical file/inline stdout, empty files and files larger than their region (byte-identical inline diagnostics), directories, missing/empty paths, missing values, malformed assignments/hex, unknown names, and all four duplicate/mixed-flag orderings. Failures return exit 1 with empty stdout; file I/O diagnostics name the flag, region and path. Unknown oracle-name wording is finding 1.
- **Baseline behavior:** independently executed **17 successful inline cases** with the archived binary and compared the candidate's exit/stdout/stderr byte-for-byte: all equal. These cover every dispatch, the witness's eight input rows, and its exhaustive step check. Evidence: `.tmp/isa-mtc-review/{baseline,candidate}-inline-results.json`.
- The committed 65,536-byte regression passes for `run-oracle` and sampled `step-check`, and actually attempts the equivalent 131,072-character inline hex argument, confirming Linux **E2BIG (errno 7)**. The expected last-byte payload is hand-derived as 165. Other committed tests assert independent expected oracle, action and classical values, beyond flag-to-flag agreement.
- `rustfmt --check --edition 2021 reference-interpreter/src/main.rs reference-interpreter/tests/cli_mem_file.rs` and `git diff --check 62a0cf0..c3714ca`: exit **0**.

The issue's original `20261101T213213.441081Z` raw run directory is absent. Used the available same published mini witness from `/home/tobias/Projects/bennett-workspace/bennett-conformance/.tmp/rung-4/runs/20261108T095801.500599Z/`: object `rung4-mini.bennett` SHA-256 **742ccbaf23ca1b89c93e8fa6e3e6953df99748d45d6ef01cfc9f91f1a89726a8**, raw `mini-region.bin` SHA-256 **b8419235dc4cfae6d5681055b9c5e6c0f43fd2bb1835a92c4ae8063fb75db959**. The bead-shaped command with `--input 4 --arg 16 --budget 400,0,512,8` now succeeds with **`{"code":0,"payload":1,"steps":71}`**, identical to `--mem` on that object. All eight inputs agree between flags; `step-check --exhaustive --arg 16 --budget 120,0,512,8` agrees byte-for-byte and reports **8 cases, exhaustive, fixed_regions=[m0]**. This is the diagnostic adapter witness, not a claim that the full SQLite rung passed.

## Scope and anti-slop

The production diff consists of usage text, one `parse_memory` helper, and replacement of the four existing memory-option branches with calls to it. File bytes enter the same containers and unchanged downstream size checks as decoded inline bytes. Interpreter semantics, published contracts, layout, budgets, and execution are untouched. One data-dispatched syntax/loader and one duplicate-name check serve both flags at every site; no old bespoke memory decoder remains. No workflow identifiers were added to product code, errors or fixtures. Scratch process IDs only provide test-directory uniqueness. No scope narrowing or promised capability rejection was introduced.

Evidence is under `.tmp/isa-mtc-review/` (`red.log`, `green.log`, `edges.log`, matching receipts, `edges-results.json` and differential results). The orchestrator's candidate full gate is separate and has no completed receipt at review time; this review does not claim its result.

Disposition: ACCEPT with P2 beads
