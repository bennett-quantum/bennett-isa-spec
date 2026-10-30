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
| Linear inrange/flags | 11 helpers; table sizes 1/2 and call R_f=4 checkpoints | widths 1–4 (two entries: 1–2) / 1,318,144 inrange + 24,704 free + 149,760 alloc + 1,123,328 call = 2,615,936 | Family commit containing this entry |
| Address dispatch/multiplexers | Not yet checkpointed | None | — |
| Row/oracle/structure | Blocked on cbk: typed wires, heterogeneous bodies, callee binding, inherited controls | None | — |

Checker integer-only invoke support deliberately provides no wire renaming
or body substitution. Such behavior is not fixed by catalog bytes under the
current contract; adding it locally would create incompatible consumers.
Native H/phase/rotation witnesses need state-vector/matrix checks: H does
not map basis states to basis states, so integer permutation checking cannot
satisfy that portion of the brief. No rows are declared inert or out of scope.
