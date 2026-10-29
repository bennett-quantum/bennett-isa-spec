# bennett-isa-conformance

The Bennett ISA conformance suites A–H (spec §15) as **data plus a
harness**. The harness is implementation-agnostic: it runs any binary
that implements the reference CLI (`check`, `hash`, `run-oracle`,
`step-check`, `run-cc`, `reverse`, `run-action`, `run-program`; see
`reference-interpreter/README.md` "Commands") and compares the
spec-defined outcome fields.

Rust 2021, standard library only. Work headless (CLAUDE.md rule 11):
`DISPLAY`, `WAYLAND_DISPLAY`, `DBUS_SESSION_BUS_ADDRESS`, `BROWSER`
unset — the harness strips them for child processes itself — and
`TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp` (created by the harness/tests).

## Run it

```sh
mkdir -p /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/.tmp cargo run --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml -- --bin /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/target/debug/bennett-isa-ref
```

`--bin` selects the binary under test; the default is
`$BENNETT_CONFORMANCE_BIN`, else
`reference-interpreter/target/debug/bennett-isa-ref`. Build that
first (`cargo build --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/reference-interpreter/Cargo.toml`) — a missing binary is a hard failure, never a skip.

Exit status: `0` when every case passes and none is unavailable, `1`
when any case fails (including an empty case directory), `2` when nothing fails but at least one case is
UNAVAILABLE. `--suite A` selects one suite, `--list` lists cases, and `--cases PATH` selects another case root. Unavailable is a loud, counted outcome, not a skip (see
below).

## Case format

Each case is a directory `cases/<suite>/<id>/` with a `case.json`
plus the fixture files it references (shared object texts live in
`cases/objects/`). Fields:

| field | meaning |
|---|---|
| `id`, `suite` | e.g. `B-fault-code-5`, `B` |
| `spec` | the spec sentence(s) the case tests, with section numbers |
| `derivation` | the hand derivation of the expected outcome from the spec — never copied from a binary's output |
| `object` | path (relative to the case dir) of the object text, or omitted |
| `command` | argv after the binary; `{object}` and `{dir}` are substituted |
| `expect.exit` | expected exit status |
| `expect.stdout_json` | expected outcome as a JSON **subset**: objects match field-wise (extra reported fields ignored), arrays match exactly, numbers as binary64 values, big integers as decimal strings (§9.1) |
| `expect.stdout_json_exact_at` | dotted paths (e.g. `state`) where the subset subtree must match exactly, no extra keys |
| `expect.stdout_json_equal_fields` | pairs of dotted paths that must be equal (e.g. a reversal's two memory digests) |
| `expect.stdout_empty`, `expect.stderr_contains` | for diagnostics (exit 1, rule cited on stderr) |
| `scenario`, `expected_outcome` | structured inputs and hand-derived semantic outcomes for staged cases; these are not a claim that a missing CLI has executed them |
| `unavailable` | `{"deliverable": N, "reason": ...}` for suites served by staged deliverables 8–11 |

### Unavailable cases

Where the reference interpreter cannot yet serve a suite (staged
deliverables: 8 §10 launch library, 9 §8 targets, 10 §11 schedule
consumer, 11 §12.1 rigorous checker — `reference-interpreter/
PROGRESS.md`), the case still exists with its derived expectation in
`expected_outcome` and the explanation in `derivation`, and the harness reports
`UNAVAILABLE <id>: unavailable: deliverable N`. When the case has a `command`, it
is a **probe**: the harness runs it and requires the loud staged
refusal the `expect` block describes. A binary that silently accepts
a probe fails the case — that is the signal to promote it to a
runnable case.

## Tests

- `tests/json_subset.rs` — the JSON subset comparator.
- `tests/case_data.rs` — every suite A–H has cases; every case cites
  a spec section, shows a derivation, and references existing files.
- `tests/harness_cli.rs` — empty directories fail; unavailable outcomes exit 2 and are counted.
- `tests/harness_fake_bin.rs` — the harness judged against a
  shell-script fake CLI (pass, fail, and probe discipline).
- `tests/reference_run.rs` — the whole suite against the binary under
  test; fails if any available case fails.
- `tests/headers.rs` — compiles `include/bennett_launch.h` and
  `include/bennett_sched.h` with `cc -std=c11 -Wall -Werror`.

The Rust tests check the harness, case integrity, C compilation and all available
reference cases. Their success does not certify staged suites; the standalone
harness returns **2** while any suite is unavailable. See [PROGRESS.md](PROGRESS.md)
for the exact verified counts and commands.

Suite C includes independently derived full truth tables for an 8-interface-bit
oracle (input, arbitrary target and fault entries), a four-bit permutation
action, a two-bit `qinplace` pair, and a three-bit borrow frame. The borrow-frame
fixture exposes its borrowed bit as an interface operand so both entry values
can be tested through the CLI; the real `qborrow` path is also tested separately.
The frame's complete basis permutation proves independence from an entangled
external reference by T3 and linearity. `step-check` cases separately compare
the sequential and reversible engines, including complete prefix reversal.
Run the same case data with `--bin PATH` for each executor to check agreement
against the hand-derived outcomes. Missing target/schedule/checker/C API
surfaces remain explicit unavailable results.
