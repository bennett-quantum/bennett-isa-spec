# Current state — 2026-11-01 06:58:38 UTC

ISA 0.9 is published alongside 0.8 as `spec/Bennett-ISA-Spec-0.9.md`.
All three council seats accepted on 2026-10-31: sol r3, codex r2/r4,
muse r5; K64/K65 ratified. Publication follows the maintainer's
2026-10-31 ruling; no council round is needed for this mechanical step.
New consumers (bennett-vm M5 onward) implement ISA 0.9.

`spec/Bennett-ISA-Spec.md` remains the unchanged accepted 0.8 (round 8),
targeted by the live dialect 0.2.x gate and the reference interpreter
until the dialect 0.3 track moves. No interpreter 0.9 admission is claimed.

The bound catalog is `spec/ref-0.9.json`: 219 recipes, envelope SHA-256
`5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197`.
The redundant draft JSON is removed. The complete worked fixture remains
at `conformance/fixtures/controlled-action-09/`. Script/test consumers use
the published names; `spec/isa-0.8-to-0.9.diff` uses the published labels.
`conformance/tests/draft_witnesses.py` retains its referenced filename.

Sibling coordination (outside this implementation): bennett-vm's
`SPEC_VERSIONS.toml` still pins the former draft spec filename; its
implementer updates it. bennett-conformance's `clauses/ISA-0.9-draft.*`
graph files belong to the orchestrator. Historical Beads records retain
old filenames. No sibling repository edits, push or remote/Dolt sync.

Publication tracking: b-isa-spec-fhp (complete). Headless conformance gate:
113 passed, 0 failed, 0 ignored. Publication witness and catalog --check:
PASS, exact diff and forbidden-word checks retained. Initial gate exposed
a stale catalog-status string assertion; corrected mechanically before
the passing full rerun. 0.8, K rows, fenced examples, printed hashes and
catalog bytes are unchanged. No reference-interpreter suite run (source
untouched). Test log: `.tmp/publication-cargo-test-final.log`.
