# Round-11 fixture comment audit

All 16 .comments files were read against their literal .bisa rows, the accepted
ISA 0.8 rules and the independent expectations in the test targets below.
A passing hash/existence check alone is not evidence of a correct hand claim.

| Sidecar | Independent hand check / executable coverage | Disposition |
|---|---|---|
| actions.comments | Copy/XOR/uncompute gives t XOR p; zero image, doubled stores/exchanges are identity; three Xs flip; H/CX Bell; lifted payload x XOR 1 in 3 rows (quantum, wide_oracle) | unchanged |
| arithmetic.comments | Arithmetic modulo 256; -7/2=-3 remainder -1; MIN/-1 fault 6; oversized shifts zero by §4.1; sext/shift/trunc 255; parallel swap; explicit branch/label and fault literals (oracle, step_machine) | unchanged |
| classical.comments | Triangular sum; inner clock segment undone once; five effect records, clock 1; absent sector restored; impure uncall, entropy, unknown-device, null/div0 faults; copied host reply truncation/full length (classical, cli_xam) | unchanged |
| control_flow.comments | Prefix flag conjunction/parity; conditional H/X amplitudes; exact six-gate XZXZ=-I; rounded θ dyadic and two nonzero interference branches; exchange, little-endian dead image, translation lemma (quantum_control, cli_xam) | corrected exact-vs-rounded claim; clarified bytes is the data symbol and dead its hex image |
| heap.comments | Base 32, 2-byte 0x1234→34 12; max(n,1) for zero alloc; pinned/free/null and capacity faults (oracle, step_machine) | unchanged |
| inc8.comments | 255+1 wraps; const/add/retval = 3 rows; B=0/2 budget faults (oracle, step_machine) | unchanged |
| mutual.comments | main has two rows, f next two, g starts at 4; main call + four g rows + retval = 6; name ordering distinct from hash ordering (identity) | unchanged |
| nested.comments | Three-row inc8; explicit fr test propagates code 9; paired XOR calls clear payload/fault even at B=0; ignore does not propagate (oracle) | unchanged |
| r9-codex-oracle.comments | Same-token self-adjoint XOR calls cancel at B=0/1; H² restores c=0; CX(0,b) identity on arbitrary bR (r1_borrow, r9_identity helper) | unchanged |
| r9-loop.comments | Each lifetime returns b under c=0 and replaces c by fresh zero; H² on t; no cancellation across invalid returns (r1_borrow) | unchanged |
| r9-sol-oracle.comments | Same direct-oracle XOR argument as codex fixture, including nonzero fault delivery (r1_borrow) | unchanged |
| r9-sol-qinplace.comments | One-row @o is identity at B>=1, but B=0 has code 1 and fails §4.5 before qinplace mutation; all t,b,R basis entries, superposition and Bell bR at B=1 (r1_fixtures, r1_borrow) | exact council replacement; matching generator branch corrected |
| registers.comments | Clean concat/views; two flips; dirty free traps; Bell 1/2 outcomes, lexicographic draws; measured segment cannot reverse (registers, cli_xam) | unchanged |
| sum_db.comments | 3+2+1=6; R_sum=3+3+1+9=16; n=16 needs 17*16=272; n=17 accesses db[16] (oracle) | unchanged |
| tiny_memory.comments | 1+8+1+4=14 interface bits; DIV0 row 4, db&1 row 8; B=9 idle; complete tuple reversal (step_machine) | unchanged |
| wide.comments | (2^100-5)(2^100+5)=2^200-25 leaves remainder 28; call/ret/retval 3 rows; 5-cell frame; high code bit 200 (wide_oracle, bitvectors, cli) | unchanged |

Generated comments: tests/support/r9_pins.py is the only Python sidecar writer.
Its sol-qinplace branch now emits the exact B>=1/B=0 distinction; direct
oracle-call cases retain valid fault-XOR cancellation and the loop branch
retains its per-lifetime derivation. r10_pins.py and r11_pins.py generate
envelopes/case records, not hand-result sidecars; both were checked for the
same class of claims. No further sidecar semantic error was found.

The same rounded-TAU-as-minus-I wording also occurred in an inline comment
in tests/quantum.rs::every_gate_and_controlled_global_phase. It now describes
the rounded residual and asserts two exact-support entries and non-basis
control, retaining the approximate diagnostic-amplitude comparison.
No runtime gate, budget, borrow or exact-state semantics changed for B03.
