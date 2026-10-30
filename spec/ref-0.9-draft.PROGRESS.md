# ref 0.9 catalog checkpoint (b-isa-spec-8r4)

Status: incomplete, not publishable; no normative catalog hash/count edit.
Contract blockers filed before implementation: b-isa-spec-cbk and
b-isa-spec-8sc. Their descriptions quote the exact §11.1 sentences and
required contract changes. Publication remains gated on accepted 0.9.
No reference-interpreter source was read or copied.

| §11.1 family, in required order | Recipes completed | Widths / tuples verified | Commit |
|---|---:|---|---|
| Copy/XOR/exchange/native | 8 (rotation/variadic/quantum subset blocked: 8sc) | 1–4 / 1,480; native / 36 | 5663477 |
| Ripple add/sub/comparisons | 8 value helpers; row binding still cbk | 1–4 / 26,880 | e332108 |
| Other arithmetic/casts/select | 24 helpers (12 width-specialized shifts); row binding cbk | 1–4 / 664,560 arithmetic + 21,760 cast/select | 65b24f5 |
| Linear inrange/flags | 11 helpers; table sizes 1/2 and call R_f=4 checkpoints | widths 1–4 (two entries: 1–2) / 1,318,144 inrange + 24,704 free + 149,760 alloc + 1,123,328 call = 2,615,936 | a0d7215 |
| Address dispatch/multiplexers | 3 two-cell helpers; region binding cbk | cell widths 1–4 / 56,160 | 08a2103 |
| Row/oracle/structure | Blocked on cbk: typed wires, heterogeneous bodies, callee binding, inherited controls | No semantic certification; integer invoke/inverse and fail-closed boundaries tested | Blocker checkpoint commit containing this entry |

Checker integer-only invoke support deliberately provides no wire renaming
or body substitution. Such behavior is not fixed by catalog bytes under the
current contract; adding it locally would create incompatible consumers.
Native H/phase/rotation witnesses need state-vector/matrix checks: H does
not map basis states to basis states, so integer permutation checking cannot
satisfy that portion of the brief. No rows are declared inert or out of scope.

Final checkpoint: **54 recipes**, **3,386,812 family witness tuples**.
Widths/configurations are precisely those in the table and README, not a
claim to exhaustive arbitrary-table/closure execution. Exact §16.3 lifted
row/frame/construction domains are not certified; those recipes cannot yet
be bound under the closed schema. Large/general configurations, aliases and
call records are not represented by the primitive helper witnesses.

Checkpoint envelope SHA-256, independently in Rust and Python -I:
`3a99ce3028e1ecc421b3991e3b5b7423ee33323badc5ca3ce1e61088a5e5055d`.
This is **not** a completed-library binding. Neither the draft's catalog
hash/count marker nor its diff was changed; byte-exact diff and unchanged
accepted 0.8 checks still run in the existing gate.

Resumption requires b-isa-spec-cbk / b-isa-spec-8sc contract repairs before
row/oracle/structure serialization, then replacing the specialized primitive
checkpoints with complete generic coverage and expanding the witnesses to
the required configurations. Do not promote any staged case or publish ref/.
The parent b-isa-spec-8r4 remains open. Unrelated concurrent edits in
reference-interpreter/ and docs/ were neither read for implementation nor
included in commits.

Final headless/offline gate: fmt --check and clippy --all-targets -D warnings
green; cargo test **40 pass / 0 fail**. Full 0.8 harness **445 pass / 0 fail /
60 unavailable (505 cases), exit 2**. Six new family cases add unavailable
only, naming b-isa-spec-8r4. No push.
