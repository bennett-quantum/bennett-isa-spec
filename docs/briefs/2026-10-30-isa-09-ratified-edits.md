# Brief: ISA 0.9 draft — apply the maintainer's ratified decisions (text pass)

Repository `/home/tobias/Projects/bennett-workspace/bennett-isa-spec`. You are the **sole editor** of the 0.9
draft for this pass. On 2026-10-30 the maintainer ratified every open 0.9 decision; the record is
`/home/tobias/Projects/bennett-workspace/bennett/docs/decisions/2026-10-30-maintainer-decision-memo.md`
(read the whole memo, then its "Ratification" section, which is the binding list). Your job is to turn each
ratified decision into normative 0.9 text, surgically, so that the draft no longer contains any
"maintainer decision required" marker or any alternative reading in normative text.

## Owned paths (write nowhere else)

- `spec/Bennett-ISA-Spec-0.9-draft.md` (surgical edits; the diff stays reviewable).
- `spec/isa-0.8-to-0.9.diff` — regenerate before every commit:
  `diff -u /home/tobias/Projects/bennett-workspace/bennett-isa-spec/spec/Bennett-ISA-Spec.md /home/tobias/Projects/bennett-workspace/bennett-isa-spec/spec/Bennett-ISA-Spec-0.9-draft.md > /home/tobias/Projects/bennett-workspace/bennett-isa-spec/spec/isa-0.8-to-0.9.diff` (exit 1 is normal).
- `spec/isa-0.9.PROGRESS.md` — append a dated "Ratified decisions applied" section (one row per decision:
  memo item, section edited, witness added, status, commit) so a restarted session resumes from it.
- `conformance/` — new witness cases for new text, as the K12/K13 job did (`unavailable pending 0.9`,
  naming the bead), with independent Rust/Python checks of every number you print.
- `bd note` / `bd create` (run `bd` from the repo root with `BEADS_HOOK_TIMEOUT=20`).

Never touch `spec/Bennett-ISA-Spec.md` (accepted 0.8), `spec/README.md`, `reference-interpreter/` (another
agent is the sole implementer there and commits concurrently; you may READ it and run its built binary),
`include/`, `docs/`, `worklog/`, `HANDOFF.md`, `CLAUDE.md`.

## Read first

`CLAUDE.md` (rules 1, 8, 10, 11); the memo; `spec/isa-0.9.PROGRESS.md` (top three sections: the K12/K13
and milestone-4 follow-ups state the proposals and the alternatives that are now rejected); the draft's §0
change list, §4.1, §5.3, §8.1–§8.3, §9.1, §9.4, §10.1, §11, §11.1, §12.1–§12.3, §17;
`../bennett/docs/council/rubric-isa.md` (what the council grades; narrowing is S0; "two conformant
implementations would differ" is the S0 test for every sentence you write).

## Edits, in order

1. **A2 LAUNCH-ESTIMATE (§10, §10.1, §12.3).** The explicit 0.9 estimate signature with the resolved
   parameter vector (`params_jcs, params_len`, `[]` only for no arguments); flat injective percent-escaped
   `(resource, class)` keys; exact reduced rationals in the certificate plus a 34-significant-digit
   upward-rounded display enclosure. Remove the alternative readings from normative text (they stay in
   the progress file as rejected). Keep the existing witness; make sure its numbers follow from the text.
2. **A3 CLAIM-REPLAY (§9.2, §12.1–§12.3).** `expected` mandatory on sampled claims (no omitted-field
   default); per-claim resolved invocation on standalone claims and `bindings.invocation` at the scheduled
   root, hashing the canonical args array with the typed encodings already drafted; tables as complete
   positional before/after tuples, one row per domain element, `program` for large domains. Same removal of
   alternatives.
3. **A1 ISA-REF-LIB (§11.1, §17 item 7).** Replace "maintainer decision required" with the decided contract:
   schema, envelope hash rule, ordinals, scratch identity, `removable` empty, deterministic Boolean
   decomposition, decomposition set = the §17 defaults made explicit. State that the complete serialised
   catalog `spec/ref-0.9-draft.json` is produced by a separate job (bead b-isa-spec-8r4) and that `ref/` is
   published after 0.9 acceptance; keep the qxor witness.
4. **A6 topology (§8.2, §8.3, §17).** Add an optional `topology` field to the target description: an
   ordered array of physical vertices (each with its addressable class), an array of undirected edges as
   vertex-index pairs (sorted, no duplicates, no self-loops), and the rule that when present, a native
   multi-wire gate of a class is realisable only on vertices joined by an edge (schemes and §11 schedules
   bind to it); absent = all-to-all (today's semantics unchanged). Add a §17 item: routing and movement
   cost under a topology is deferred to 0.10, default "no routing cost is charged and no routing is
   inserted; a schedule that needs a non-adjacent pair is E_SCHEDULE_INFEASIBLE naming this item". Add one
   witness target (a 2x2 grid) with its canonical bytes and SHA-256, computed independently (Python JCS),
   as a conformance case. Make sure JCS canonical bytes have no trailing LF (see b-isa-spec-1pl).
5. **B1 multi-block actions (§5.3).** One sentence: an action body with several `block`s is executed as its
   blocks in text order, each a unitary fragment; labels are descriptive and carry no control semantics;
   lifetimes may span blocks. Confirm this agrees with how `reference-interpreter` treats them (read its
   README/SPEC-QUESTIONS and run the binary on a two-block action); if it does not, say so in the progress
   file and file a bead rather than silently choosing.
6. **B5 frame schema (§9.4, §12.3, §9.1).** Publish the JSON encoding of a frame as typed records: `slots`
   (interface slots with identities), `scratch` (per class, per cleanliness, widths as expressions in `c`),
   `history`, `borrow`, `ancillae` (scheme ancilla frames); unknown fields rejected per §9.1; expressions
   restricted to `c`; shared lifetimes disjoint. One witness frame with its canonical bytes.
7. **A5 draw ordering (§4.1).** State the draw-consumption order normatively, exactly the convention the
   reference CLI implements (read `reference-interpreter/README.md` and the `--draws` handling; cite the
   bead b-isa-spec-bas in §0). Add a witness with two measurements and two draws whose result differs
   under the other order.
8. **A4/A5 confirmations.** Q14, EPS-DECADE, BLOCK-IMAGE, ISA-7: mark decided in §0 and the progress file;
   no text change unless a sentence still reads as a proposal.
9. **§0 change list and §17**: every edit above gets its §0 row (K-number, sections, beads); every deferral a
   §17 item with a default. Update `reference-interpreter/SPEC-QUESTIONS.md`? No: that file is not yours;
   record in the progress file which SPEC-QUESTIONS entries the new text closes.

Hashes of 0.9 examples stay placeholders (Q12, ratified) except the independently computed JCS target/frame
witnesses, which you compute and check with Python (`-I`) and a Rust test in `conformance/`.

## Gate

Headless, offline, project-local TMPDIR (`mkdir -p .tmp` first; `env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp`):
`cargo test --offline --manifest-path /home/tobias/Projects/bennett-workspace/bennett-isa-spec/conformance/Cargo.toml`
must stay green; the full 0.8 harness count (currently 445 pass / 0 fail / 48 unavailable) may only grow in
`unavailable` by your new pending-0.9 cases, never in fail. Record the exact observed counts.

## Process

- Commit owned paths after every numbered edit, never a red tree:
  `BEADS_HOOK_TIMEOUT=20 git -C /home/tobias/Projects/bennett-workspace/bennett-isa-spec commit -- spec/Bennett-ISA-Spec-0.9-draft.md spec/isa-0.8-to-0.9.diff spec/isa-0.9.PROGRESS.md conformance`
  (retry on `index.lock`; the interpreter agent commits `reference-interpreter/` concurrently, so never
  `git add -A` or commit without a path list). Do not push. No `cd` in compound commands; absolute paths.
- Rule 10: nothing you write may reject, refuse or omit a north-star capability; a deferral states a default
  that keeps the capability reachable.
- Final message: commit hashes, the per-decision table (memo item, section, witness, status), the conformance
  counts before/after, every place where the ratified wording could not be applied verbatim and why, and any
  disagreement you found between the reference interpreter and the new text.
