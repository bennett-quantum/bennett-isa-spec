# ref 0.9 catalog checkpoint (b-isa-spec-8r4)

Status: incomplete, not publishable; no normative catalog hash/count edit.
Contract blockers filed before implementation: b-isa-spec-cbk and
b-isa-spec-8sc. Their descriptions quote the exact §11.1 sentences and
required contract changes. Publication remains gated on accepted 0.9.
No reference-interpreter source was read or copied.

| §11.1 family, in required order | Recipes completed | Widths / tuples verified | Commit |
|---|---:|---|---|
| Copy/XOR/exchange/native | 8 (rotation/variadic/quantum subset blocked: 8sc) | 1–4 / 1,480; native / 36 | First family commit containing this entry |
| Ripple add/sub/comparisons | Not yet checkpointed | None | — |
| Other arithmetic/casts/select | Not yet checkpointed | None | — |
| Linear inrange/flags | Not yet checkpointed | None | — |
| Address dispatch/multiplexers | Not yet checkpointed | None | — |
| Row/oracle/structure | Blocked on cbk: typed wires, heterogeneous bodies, callee binding, inherited controls | None | — |

Checker integer-only invoke support deliberately provides no wire renaming
or body substitution. Such behavior is not fixed by catalog bytes under the
current contract; adding it locally would create incompatible consumers.
Native H/phase/rotation witnesses need state-vector/matrix checks: H does
not map basis states to basis states, so integer permutation checking cannot
satisfy that portion of the brief. No rows are declared inert or out of scope.
