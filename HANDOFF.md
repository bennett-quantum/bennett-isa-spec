# Current state — 2026-11-01 12:08:06 UTC

Pass X / b-isa-spec-5cd is implemented and remains IN_PROGRESS FOR COUNCIL
ROUND 8. The one-reviewer round covers passes VIII, IX and X together;
9eq/K78 and 7js/K79 retain their pending review status. No council run was
started by this implementation sub-agent. The orchestrator owns the council
run, session worklog and downstream statement-layer regeneration.

Reading A is adopted for recompute: the entire replacement's caller leaves
and call edges belong to the carrier's coordinates, with one fresh caller
leaf space and independent semantic-call space. Its optional rebinding of
reverse is declined: written inverse rows and the final release retain
source points and per-point spaces. K65 supplied carrier/selection but not
multirow replacement ownership; K64 and §11.1 did not fill that gap.
This is a real ambiguity, not a not-a-defect closure.

New §11 Cleanup leaf ownership paragraph: one MUST, K80 only in §0;
K1–K79 byte-identical to ba7d13f. Fresh uncomputation has separate call
occurrences from fresh computation. Callees retain their own spaces under
fresh edges; raw inverse retains paths/ordinals/call occurrences/scratch
identities while existing §11 step/phase semantics still apply. K62 source
adjoints and K64 support partition remain unchanged.

The six-row witness has reverse leaves at entry:3/4/5, each ordinal 0;
recompute has eight caller leaves at entry:5 ordinals 0–7, including fresh
u and the final release d. Replacing the two computing CXs by semantic
one-CX calls yields fresh carrier call_index 0/1 then 2/3, callee ordinal 0,
with caller allocation/XOR/releases at 0/1/2/3. Original computation calls
stay at entry:1/2, each call_index 0. Source selectors and a duplicate
fresh-gate tie are tested with the reference selector engine.

Reference src was read only. It still targets/adopts 0.8, and has no general
0.9 multirow cleanup source builder. Three finite tests in
reference-interpreter/tests/cleanup_ownership.rs witness literal reverse
source/adjoint behavior and catalog-constructed recompute. The adapter
explicitly supplies source points and serialized call_index; opaque
Expander invocation IDs are not advertised as K63 serialized indices.
Primitive/call source witnesses pass CLI hash/check; basis and coherent
source cases restore complete state and allocator checkpoint. Constructor
cases cover all x/z/control basis inputs, clean releases, full live-state
phase and raw reversal. Low-level allocation IDs remain monotonic; the raw
harness compares live amplitudes and wire count, not allocator rollback.
No red-to-green src fix or full 0.9 executor admission is claimed. Initial
harness syntax/type/allocator/control-arity errors were corrected; an
ordinary-source spelling of the recompute circuit was not used as an
admission claim after its §6.2 causal-check rejection. No fixture edit,
removal or weakening occurred.

Required gates: all exit 0, all headless (DISPLAY, WAYLAND_DISPLAY,
DBUS_SESSION_BUS_ADDRESS and BROWSER unset; TMPDIR set to this repo's .tmp).
Both Cargo commands used --offline and the requested manifest paths.
- reference-interpreter: 488 passed / 0 failed / 5 unchanged ignored
  (485 before this pass plus three regressions).
- conformance: 113 passed / 0 failed / 0 ignored.
- python3 -I conformance/tests/draft_witnesses.py: PASS on final spec/diff.
- python3 -I conformance/tools/ref_catalog.py --check: PASS, 219 recipes.
Logs: .tmp/pass-x-{reference,conformance,witness,catalog}.log;
initial/final targeted evidence: .tmp/pass-x-before-text.log and
.tmp/pass-x-regression.log. Rustfmt and owned-path whitespace checks pass;
the generated diff is independently byte-exact, with its context spaces.

Review brief: docs/briefs/2026-11-01-isa-09-passes-viii-x.md.
Exact wording, dispositions and evidence: top of spec/isa-0.9.PROGRESS.md.
Implementation commits: 0f4e31c and a264277 (regressions), 22687a9 (K80,
witness, diff and brief), f3894f7 (inverse phase clarification).
Pre-records HEAD: f3894f798cd0798b817cea0709af12527b4c967f. This handoff, final PROGRESS and exported Beads
are committed next by explicit path list. The final report gives that HEAD.

Frozen accepted 0.8 SHA-256: 90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e.
Published catalog bytes SHA-256: 52dbc3f8f0c91d587fd933a43080385fd5284abcfec0ce84788229fcb15c53d8.
Catalog envelope: 5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197.
Final 0.9 spec SHA-256: 6dd61f1c1cb58130300deaa813ef51a165ed49a1ec3ae792ece058a606a4fd79.
Exact-label diff SHA-256: 8f4be5569576aab8398952874823ab25d254e87afa42a1b8aacf104d2a2138cd.

No push, remote/Dolt sync or sibling repository write. Prior n8j/pass-IX
evidence remains in their PROGRESS files and baseline ba7d13f. Preserve the
pre-existing untracked .tmp/, including this pass's logs. After the records
commit, only that untracked .tmp/ should remain in git status.
