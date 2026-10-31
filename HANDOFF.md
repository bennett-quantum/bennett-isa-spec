State (2026-10-31 01:10Z): reference-interpreter/ at **353 tests + 5 sweeps green** after rounds 9-11; **round 12 on
09fc712: sol core ACCEPT**, astra revise; two blockers left (lossless layout leaves for colliding dotted paths; interface
region names shared with function symbols), brief docs/briefs/2026-10-31-isa-ref-round-12-fixes.md, editor launch
waiting on Codex pace. Main still backed up on origin/wip/ref-library-2026-10-30. Read ../bennett/HANDOFF.md first.

State (2026-10-31 ~22:26Z, resumed evening session): reference-interpreter/ at **338 tests + 5 sweeps green** after
council rounds 9-10 (one sol editor each; briefs docs/briefs/2026-10-30-isa-ref-round-{9,10}-fixes.md); **round 11
prepared** on snapshot 45475a1 (worktree ../bennett-isa-spec-review, packet ../bennett/docs/council/isa-ref-2026-10-29/
packet-round-11.md), sol reviewing; astra next, then aggregate; the engine drew no new finding in round 10, so an accept
is plausible. Ref library job still paused at fcea698 (held for pace); main still on origin/wip/ref-library-2026-10-30.
Read ../bennett/HANDOFF.md first.

State (2026-10-30 ~14:30Z, maintainer pause): reference-interpreter/ at 315 tests + 5 sweeps green after council
rounds 6-8 (one sol editor per round; round 8 completed the per-wire fact-transfer engine); **round 9 prepared** on
snapshot 4796970 (worktree ../bennett-isa-spec-review, packet ../bennett/docs/council/isa-ref-2026-10-29/packet-round-9.md,
sol reviewing at the pause; astra and the aggregate are next; muse's seat held until 2026-10-31). 0.9 draft carries every
maintainer-ratified decision as normative text with conformance witnesses (0b913da). Ref library: 54-recipe checkpoint
(spec/ref-0.9-draft.json, hash 3a99ce30...055d) and a stopped job extending §11.1 (typed formals/substitution, angles,
variadic wires; beads cbk, 8sc) at fcea698 with a RED test in conformance/tests/ref_call_flag.rs, so `scripts/pre-push`
is red and main is backed up on origin/wip/ref-library-2026-10-30; resume from spec/ref-0.9-draft.PROGRESS.md with
docs/briefs/2026-10-30-ref-library-contract-completion.md. Conformance harness: 445 pass / 0 fail, unavailable growing
only by pending-0.9 witnesses (each case.json carries `unavailable.reason`). Read ../bennett/HANDOFF.md first.

State (2026-10-29, session end): spec/Bennett-ISA-Spec.md draft 0.8 accepted by the council (2026-10-28, round 8); settled. reference-interpreter/: under council review (run ../bennett/docs/council/isa-ref-2026-10-29, members fable/astra/muse, rubric-impl; tools/review_packet.py + a detached worktree ../bennett-isa-spec-review per round). Round 1 (snapshot 3dea965): no consensus, 30 blockers. MILESTONE 1 REACHED at e8629de (Opus implementer; 17 commits): all 30 resolved, 2 declined with the deciding spec sentence (muse B4: an `in` wire is a qstore value per §4.6/§5.1; B16 in part: no variant enum exists), B19 partial (launch §10, targets §8, schedule consumer §11, rigorous certificate checker §12.1 staged as deliverables 8-11 in PROGRESS.md). Verified by the orchestrator: cargo test 152 passed / 0 failed / 5 ignored; the agent reports all 5 ignored sweeps green (4 min 57 s), clippy -D warnings and fmt clean. Response file for the council: reference-interpreter/council/response-round-1.md. SPEC-QUESTIONS 14-27 added, 4/6/11/12 revised, each with a proposed 0.9 wording. The codex job isa-ref-interpreter is STOPPED for good (status.json stopped-by-maintainer); the Opus agent's brief is docs/briefs/2026-10-29-isa-ref-council-fixes.md.
NEXT (not started; session wound up by the maintainer): (1) round 2: `git -C . worktree remove --force ../bennett-isa-spec-review; git worktree add --detach ../bennett-isa-spec-review e8629de`, build + test there, `python3 ../bennett/tools/review_packet.py --root ../bennett-isa-spec-review/reference-interpreter --commit e8629de ... > ../bennett/docs/council/isa-ref-2026-10-29/packet-round-2.md`, copy reference-interpreter/council/response-round-1.md to ../bennett/docs/council/isa-ref-2026-10-29/round-1/response.md, then council prepare/run/aggregate round 2 with the same options as round 1 (the diff from round 1 is attached automatically); (2) deliverables 8-11 and the outstanding list by the same implementer; (3) b-isa-spec-xam CLI commands (run-cc, reverse, run-action, run-program) for the dialect precheck.
Previously: Then: b-isa-spec-xam (CLI run-cc, reverse, run-action, run-program; the dialect 0.2 precheck job dialect-check-0.2 fails loudly on them until they exist); push main once cargo test is green.
ISA 0.9 DRAFT DONE (2026-10-29, Opus, commit 9861107): spec/Bennett-ISA-Spec-0.9-draft.md (2,532 lines) next to the untouched 0.8, spec/isa-0.8-to-0.9.diff (-156/+950), spec/isa-0.9.PROGRESS.md (decisions, derivations D1-D18, placeholder list, open questions Q1-Q14). Nine examples with hand-derived results; every hash is a unique placeholder until a 0.9 precheck exists (the binary hard-codes 0.8). Grading gate (maintainer): reference interpreter council-accepted AND dialect 0.2 through round 1. Open for the maintainer: Q14 = bead 7u6 (lprocs cannot pass tokens or classical args to lifted calls; rule 10 says add the capability: lproc token/classical params or a readable classical-argument environment); Q1-Q13 recommended defaults are in the progress file. Beads filed by the draft job: 8wn, exc, 04w, 7u6.
Previously: ISA 0.9 for BLK-1..BLK-4 (../bennett-dialect-spec/spec/outline-0.2.md §25) plus wording items and evidence-backed I1-I8; issues in bd (bd ready; label isa-0.9-candidate).
Read first: ../bennett/HANDOFF.md, then docs/notes/2026-10-28-budgets-loops-variable-time.md.
Remote: github.com/bennett-quantum/bennett-isa-spec (main gated by scripts/pre-push via .beads/hooks/pre-push; wip/* branches are ungated backups).
See worklog/001_2026-10-28_bootstrap.md.
