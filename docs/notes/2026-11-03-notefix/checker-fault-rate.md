# Checker fault rates — oracle-only, invocation-bound recounts

Full round-4 revision for council review, 2026-11-02; checker request 3
in the existing cross-repo filing queue. Supersedes
`2026-11-02-rev/checker-fault-rate.md`; proposal only. This document
retains the complete accepted recount contract and replaces its unsupported
artifact-transport premise.

## Defect classification and North Star scope

ISA 0.9 §12.3 names fault_rate.domain, faults_by_code and two exclusion
counters but provides no invocation binding for them. K13 binds claims
individually and scheduled certificates at the root; several claims or
callee contexts can use different args/budgets. §§5.2/14.1 makes F_B
depend on those values. A flat recount is therefore ambiguous. This is
an S0 contract gap, not a change to oracle execution.

A separate gap is explicit here: §§12.1–12.3/14.1 name exclusion counters
but do not define independently replayable exclusion evidence. The first
note's assertion that this validation was already specified was wrong.
The repair below supplies that obligation and a concrete default.
North Star §§3.2/5.1 separates native-comparison exclusions from reference
validation and records source-definedness assumptions; post-optimization
classifiers do not detect source UB already erased by optimization.

North Star §5.1 requires fault-rate reporting and §8 requires zero on
its acceptance domains. Dropping rates, choosing a convenient budget,
removing excluded inputs from reference validation or treating unchecked
counts as verified would not meet that requirement. Actions remain
supported: §5.3 gives them no fault outcome of their own, so this repair
uses an explicit non-applicable value for their records.

## Alternatives and concrete witnesses

| Choice | Consequence |
|---|---|
| Take the first claim's invocation | Ambiguous and dependent on claim ordering. |
| Restrict a region to one invocation | Removes existing composition capability. |
| Bind every oracle recount explicitly | Recommended; exact standalone invocation or full scheduled context. |

The oracle has x:wire[2], target:wire[2], fault:wire[4], a:i2, and body
`r = lift.add x a; lift.retval r`. At args=["1"], budget
["1","0","0","0"] cannot reach retval: all 4×4×16=256 tuples contribute
code 1. Budget ["2","0","0","0"] succeeds on all 256 with payload
(x+1) mod 4 and code 0. Every tuple includes its arbitrary target and
fault words. Incoming f=1 becomes zero on a code-1 failure; success
leaves it one. The final accumulated fault word is not the fault event.

For a classification-count fixture flagging precisely x=0, 64 of those
256 tuples are excluded from native comparison. Even then B=1 reports
faults_by_code={"1":"256"}, with undefined_excluded="64" separately.
This fixture demonstrates counting and evidence replay; it does not
assert that the two-row reference add itself has source UB. A real UB
classifier must be identified with its materialized program/configuration.

The published controlled-action-09 certificate has `controlled` and
`flip` records of kind perm_action. Neither has an F_B. The corrected
representation puts fault_rate:null on both, without evaluating an
invented action oracle. An action containing the two oracle calls above
also uses null, while its callee records retain both bound recounts.

## Recommended exact spec changes (§12.3)

Replace verification.fault_rate's schema and add the sibling context
field inside verification. The notation RATE abbreviates this record:

```text
RATE = {invocation:{args,budget},domain,faults_by_code,
        undefined_excluded,layout_dependent_excluded,exclusion_evidence?}
verification.fault_rate: null | RATE
verification.fault_rate_contexts?:[{context_key, ...RATE}]
```

Add the following normative rules:

```text
F_B recounts and fault_rate_contexts apply only to kind=oracle records.
<!-- muse-fault-rate-paragraph -->
verification.fault_rate is required and has type null | RATE. For action and perm_action records it MUST be null and fault_rate_contexts MUST be absent. For an oracle record it MUST be a RATE whenever a fault census is asserted or is required to discharge an executable or scheduled acceptance claim; it MAY be null only when the record asserts no fault census, and null discharges no fault-rate obligation.
<!-- /muse-fault-rate-paragraph -->

For kind=oracle, verification.fault_rate MUST be a RATE whenever a fault census is asserted or is required to discharge an executable or scheduled acceptance claim (including every oracle in the complete invocation tree of a bound schedule, via fault_rate_contexts); it MAY be null only when the record asserts no fault census, and null discharges no fault-rate obligation and satisfies no zero-fault gate.
For action and perm_action, verification.fault_rate MUST be null and
fault_rate_contexts MUST be absent. Null denotes no action-owned fault
outcome, not a verified zero-fault oracle. Each contained oracle
invocation remains subject to the recount obligations in its own oracle
record; no action placeholder discharges those obligations.

Every oracle RATE binds its own invocation using K13's exact typed args,
four-component budget, token-tuple encodings and validation. It inherits
no claim's invocation. Missing or conflicting bindings reject a requested
verified recount before replay; old unbound data cannot discharge it.

The rate is intrinsic to the underlying oracle's §5.2 F_B. Its domain
uses §12.2 resolution, validity, omission-at-zero and enumeration for
that oracle with zero inherited controls. Targets, fault words, inputs,
region cells and declared borrowed components each contribute as the
domain specifies; inherited control wires add no components to this
intrinsic recount. A predicate's basis value does not suppress counts.
The report is not the probability that a root execution reaches a call.

Each enumerated tuple contributes exactly once to faults_by_code[c]
when code(F_B)=c and c is nonzero. The final XOR-accumulated fault word
is never the predicate for this count. All tuples in the declared domain
are recounted, including native-comparison exclusions. Counts use
canonical nonnegative decimal integer strings; omit zero fault-code
entries. Code keys denote nonzero valid fault codes. Each count and the
sum of fault counts are at most n, the exact enumerated domain size.
Exclusion counts are separate, not subtractions from n or faults_by_code.
If limits prevent a legal recount, report unchecked (host limit),
discharging no verified fault-rate obligation; do not shrink the domain,
substitute partial counts, or declare the input domain invalid.
```

### Complete contexts and deterministic compatibility projection

```text
For an oracle present in the complete §12.1 invocation tree of a bound
schedule, including default-covered and predicated-off invocations,
report exactly one fault_rate_contexts record per context when there is
more than one. Use increasing bytewise JCS context-key order. Every
explicit invocation MUST equal that context's resolved args and budget,
including nested token tuples. Equal tuples with different process-local
token ids denote the same recount; the ids are not certificate values.

The required legacy fault_rate is the first canonical context record
with context_key removed. It MUST equal that projection and discharges
only that context. For one context the legacy RATE suffices and MUST bind
that context; an optional singleton context array, if supplied, must
match it. Missing, duplicate, reordered, foreign or conflicting context
records reject the requested report. A standalone oracle RATE binds its
own invocation and domain, with no invented root-context inheritance.
```

This keeps the required old field meaningful; a later arrays-only
interchange version could remove the projection. Claim reordering has
no effect. Full context identity, not merely (variant, control count),
distinguishes different nested token budgets and call occurrences.

### Exclusion evidence and default validation

Add this schema for RATE.exclusion_evidence:

```text
{ materialisation_hash, reference_program_hash, native_build_hash, native_entry,
  invocation:{args,budget}, domain_hash,
  classifiers:[{category,artifact_hash,version,configuration,input_map_hash}] }
```

```text
Exclusion counters MUST be independently reproduced from classification
evidence bound to the same materialized program, native build,
invocation and complete declared domain. materialisation_hash hashes
UTF-8 JCS of the recorded materialisation configuration;
reference_program_hash uses §9.2's oracle interface hash; the containing
region's closure hash and certificate object binding additionally identify
the linked reference program whose classifications are replayed;
native_build_hash identifies the corresponding comparison build, and
native_entry explicitly selects its exported comparison entry/thunk.
domain_hash hashes UTF-8 JCS of the exact domain record, without LF.
Invocation equality uses K13's resolved encodings.

Each category is undefined or layout_dependent. artifact_hash identifies
replayable classifier bytes, version and configuration identify its
semantics/options, and input_map_hash identifies the replayable mapping
from a full interface tuple to native/classifier inputs. All hashed
artifacts must be available by content for a checked result. Multiple
classifiers in one category combine their flagged sets by union; a tuple
counts once per category. Categories may overlap, so their counts must
not be added without computing their union for a native exclusion set.

Default validation replays the identified classifiers and input mapping
on every domain tuple and recomputes both counters independently. When
initial target/fault values do not feed the native program, mapping may
ignore them, but their distinct tuples still each count. A classification
flag reports an exclusion under that declared classifier; it does not
prove complete detection of source-level UB or erase the recorded
definedness assumption. This does not change F_B semantics.

Missing replayable evidence or unsupported classifier semantics makes
the exclusion report unchecked and discharges no exclusion-dependent
acceptance obligation; it MUST NOT mean zero exclusions. This includes
an asserted zero count. A complete F_B recount may still be verified
separately. Evidence that replays to different counts or has conflicting
bindings rejects the requested exclusion report. This is a new validation
contract, not a premise supplied by the pre-repair counter names.
```

ISA 0.9 defines no certificate-attachment transport. This repair introduces
a separate checker input bundle containing the certificate and the required
content-addressed artifacts. By default, classifier and input-map blobs
are supplied outside the ISA object and certificate schemas; artifact_hash
and input_map_hash are SHA-256 of their exact raw bytes. The checker verifies
those digests before replay. Missing required blobs make the exclusion
report unchecked; mismatched digests reject it. Other transports MAY supply
the identical addressed bytes. This bundle is a new checker input contract;
it adds no attachments field to either closed schema. A producer's flag list
alone is not replay evidence. All materialisation, reference-program/closure,
native-build/entry, invocation and complete-domain bindings above still apply.
Native comparison still excludes flagged inputs as the North Star
requires, while reference validation keeps every domain element.

## Mixed fat object (bennett-4ht)

Incorporate [the shared mixed-fat-object scoping statement](../2026-11-02-rev2/fat-object-scope.md#citable-scoping-statement),
including its explicit remaining launch and host-memory obligations.
Add to §§9/10/12: "A fat native binary may embed VM objects only for
marked modules. The launch manifest selects the marked VM entry, with
its actual args, budget, token tuples and image; the certificate covers
that VM closure's oracle contexts. Unmarked host main, argv/envp and
libc are not invented oracle records and need no F_B recount. Classifier
and native-comparison artifacts may be thunks calling the corresponding
materialized marked native entry with captured host state and input
mapping; native_build_hash identifies that build/entry. They need not
translate or replay the entire native binary as a VM object. Required
marked dependencies and runtime shims are recorded; host device handles
remain process-local, with only resolved shapes/tuples entering bindings."

Separate host launches of the same marked arithmetic module under B=1 and
B=2 use separate root schedule/certificate bindings and rates. The native
host binary stays native. fault_rate_contexts enumerates calls within one
bound root schedule: a device action wrapper containing both calls instead
supplies two contexts in that schedule, with distinct call-occurrence keys.
A native comparison of successful inputs uses the corresponding arithmetic
thunk, not main's exit code. Root-relative contexts never identify native
process entry. Each embedded device object/root has its own certificate
association; the external checker bundle associates artifacts with that
certificate's device closure. It introduces no whole-host certificate.

## Round-1 blocking items and resolution

| Item | Change and concrete witness |
|---|---|
| sol FR-1 / codex CFR-1: F_B applied to actions | Oracle-only recount/context rules; null for both published perm_action records. Nested oracle counts remain required. |
| codex CFR-2: unsupported exclusion premise and ambiguous recount domain | Explicit new hash-bound classifier replay default; reference recount retains all 256 tuples even with 64 separately excluded. Unavailable validation is unchecked, never assumed zero. |

## Round-2 blocking items and resolution

The [round-2 report](../../../../bennett/docs/council/contract-checker-fault-rate-2026-11-02/round-2/report.md)
and out-sol.json/out-codex.json agree that FR-1, CFR-1 and CFR-2 are resolved.
Both accept core scope; sol blocks only CFR-3. The convenor table calls the
second seat astra, while the stored report/output identifies it as codex;
this revision follows the actual outputs and claims no new council acceptance.

| Item | Exact resolution and concrete witness |
|---|---|
| sol CFR-3 — nonexistent existing attachment transport | The replacement §12.3 paragraph explicitly introduces the external checker input bundle and raw-byte SHA-256 validation. check_rev4.py supplies both blobs and reproduces 64 exclusions; removing either leaves exclusions unchecked while the independent B=1 recount remains {"1":"256"}; replacing either addressed blob with changed bytes rejects before replay. |

No prior recount or evidence-binding rule is removed. Oracle RATE typing,
reference interface versus closure binding, and separate-launch context
wording also incorporate the round's nonblocking clarifications.

## Hashes, publication and consumers

Recommend an authorized 0.9 K certificate repair. New invocation,
context, null and exclusion-evidence bytes change certificate identity.
Under an unchanged 0.9 semantic envelope, §9.2 leaves source/interface/
closure/object, target, schedule and ref catalog identities unchanged.
A full-version publication instead requires regenerating version-envelope
identities and dependent pins. No publication is performed here.

Producer, classifier harness and checker must implement the same binding
and context rules. Zero-fault rung gates select the actual oracle domain
and invocation; no budget result covers another by guessing. Monotonic
success would require its own proved statement. The existing cross-repo
request remains open for acceptance/implementation; this notes-only job
does not close it or mutate tracker/council files.

## Conformance vectors and verification

Require both 256-tuple budgets, all incoming fault values, B=0, reordered
claims, a mismatched args vector, missing budget, equal token tuples with
different ids, default/predicated-off contexts and two call occurrences.
Duplicate/omit/swap a context, conflict with its invocation or alter the
legacy projection: reject. Test overlapping classifier categories,
missing evidence (including zero counters), stale artifact/domain/native
bindings and evidence that flags 64 tuples without shrinking F_B counts.
An action without oracle callees has null; one with oracle callees still
has null and separately checked callee recounts.

This script checks exact finite reference arithmetic and the counting
witness by hand-model replay; it does not run a certificate or exclusion
checker and does not establish a real source-UB classification.

```python
from collections import Counter
import itertools
import json
import hashlib

rows = list(itertools.product(range(4), range(4), range(16)))
assert len(rows) == 256

def outcome(x, a, b):
    # Two reference rows: add, then retval. No memory or earlier fault.
    return (((x+a) % 4, 0) if b >= 2 else (0, 1))

for b, expected in [(0, {1: 256}), (1, {1: 256}), (2, {})]:
    counts = Counter()
    for x, t, f in rows:
        payload, code = outcome(x, 1, b)
        if code:
            counts[code] += 1
        after = (x, t ^ payload, f ^ code)
        assert (after[0], after[1] ^ payload, after[2] ^ code) == (x, t, f)
    assert dict(counts) == expected
assert (1 ^ outcome(0, 1, 1)[1]) == 0
assert (1 ^ outcome(0, 1, 2)[1]) == 1
# Synthetic replayable classifier fixture; not an assertion about add UB.
flagged = [row for row in rows if row[0] == 0]
assert len(flagged) == 64
assert sum(outcome(x, 1, 1)[1] != 0 for x, t, f in rows) == 256
# The exclusion predicate duplicates across 4*16 initial target/fault values.
assert len({x for x, t, f in flagged}) == 1
counters = {'faults_by_code': {'1': '256'}, 'undefined_excluded': '64',
            'layout_dependent_excluded': '0'}
assert int(counters['faults_by_code']['1']) == 256
# Null is distinct from an oracle's verified zero-fault record.
action_record = {'kind': 'perm_action', 'verification': {'fault_rate': None}}
assert json.loads(json.dumps(action_record))['verification']['fault_rate'] is None
assert 'fault_rate_contexts' not in action_record['verification']
zero_oracle = {'invocation': {'args': ['1'], 'budget': ['2','0','0','0']},
               'faults_by_code': {}}
assert zero_oracle is not None
# Binding changes cannot accidentally reuse a domain digest.
a, b = {'bundles': [0, 1]}, {'bundles': [0]}
def digest(d):
    return hashlib.sha256(json.dumps(d, sort_keys=True, separators=(',', ':')).encode()).hexdigest()
assert digest(a) != digest(b)
print('PASS fault-rate: 768 oracle tuples; 256 faults with 64 separate exclusions; action null distinction')
```


## Round-4 executable witness and evidence boundary

The files [checker-fault-rate.bisa](../2026-11-02-rev4/checker-fault-rate.bisa) and
[oracle_witness.rs](../2026-11-02-rev4/oracle_witness.rs) exercise this exact two-row oracle
through the read-only reference interpreter library. For B=0,1,2 and each
x=0..3, the witness compares the independent reference outcome with the
literal reversible machine, calls its actual commit method for all 4 target
words and 16 incoming fault words, and then inverses every executed step.
It checks the entire initial machine snapshot and clean frames. This covers
768 full interface tuples: 256 faults at each short budget, zero at B=2.

[check_rev4.py](../2026-11-02-rev4/check_rev4.py) supplies two fixed, serialized raw blobs in
an external in-memory bundle: a version-1 synthetic classifier selecting
mapped x=0, and a version-1 input map selecting tuple component 0. Their
literal bytes and raw-byte digests are present in the checker. Replaying
all 256 tuples yields undefined_excluded="64" and
layout_dependent_excluded="0". Missing classifier, missing map and an
asserted-zero count with missing evidence are unchecked. A digest mismatch
in either blob rejects before classifier interpretation; even JSON whitespace
changes fail under the old digest. A replayed count of 63 or conflicting
invocation/domain/program binding rejects. Unsupported classifier semantics
is unchecked. All these cases preserve the independent F_B recount.

This is a finite contract witness, not a production certificate checker,
a native build, a proof of source UB detection, or a complete serialized
multi-context certificate. The latter remains the conformance publication
obligation described above: concrete §12.1 keys, B=1 map {"1":"256"}, B=2
map {}, exact first-context legacy projection, and rejection on missing
contexts or a corrupted projection. It is not claimed implemented here.
The synthetic evidence bindings identify only this fixture's recorded
configuration; no real native-comparison certification is asserted.

Run the focused build and witnesses from any working directory:

```sh
timeout 300s python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-notefix-s4/run_witnesses.py
```

The script unsets GUI/browser variables, sets TMPDIR and CARGO_TARGET_DIR
under .tmp/sol-notefix-s4, bounds processes at 6 GiB, uses -j4 and applies
timeouts. It runs no full gate or git command and changes no interpreter
source. It runs the retained finite CFR-3 checker in memory against this
revision, redirecting only the job artifact paths. The current run exited
0: all 768 oracle tuples and classifier-bundle positive/negative cases
passed. Historical outcomes remain in
[the shared verification results](../2026-11-02-rev4/verification.md). The earlier inline
hand model remains an independently understandable counting example; the
actual interpreter/machine witness above supplies execution evidence.

## Muse blocking items and resolution

This joint revision starts from the latest graded notes: 0jq round 2
(`19ffcda5e44a87d0878c89e357fa4ccea163d50883e031aff781fb3e43bd923e`)
and checker-fault-rate round 3
(`71a74c494872f94fa40271dd0d81ef140fa01d6095006ad859400a5b5037a189`).
It proposes resolutions, not council verdicts. Both notes carry the same
Muse paragraph above and the same conditional oracle sentence, copied
verbatim from [out-muse.json](../../../../bennett/docs/council/accept-muse-batch2-2026-11-02/round-1/out-muse.json).
RATE abbreviates `{invocation:{args,budget},domain,faults_by_code,
undefined_excluded,layout_dependent_excluded,exclusion_evidence?}`.
A non-null RATE binds its own exact invocation and full oracle domain;
it never inherits a convenient claim's budget. For a bound schedule,
every complete-tree oracle context, including defaults and predicated-off
calls, needs its recount; the legacy field is the first canonical
context's projection. Exclusions do not shrink the reference census.
The complete context and evidence contract is retained in
[checker-fault-rate.md](checker-fault-rate.md); these requirements neither
invent a census for a structural-only packet nor relax acceptance gates.

| Blocking item | Resolution and concrete witness |
|---|---|
| 0jq-M1 | The closed schema requires `fault_rate: null | RATE`; actions use null and omit `fault_rate_contexts`. Both action fault-rate placeholders become null. A missing key is malformed, and null proves no zero-fault result. The reference_program_hash definition and owner-local inverse-pair rules are unchanged. |
| checker-fault-rate-M1 | Oracle RATE is conditional on an asserted/required census, including every oracle context in a bound schedule. A structural-only oracle may use null; it discharges no census or zero-fault gate. All accepted recount, deterministic context projection and classifier-bundle rules remain. |

There are no other Muse blocking items for these two proposals. The
accepted mixed-fat-object scope, hash conditions and prior-round fixes
remain. Relative witness links now point to their original directories;
historical verification reports above remain historical.

### Structural-only oracle conformance vector

Take a standalone oracle with body `r = lift.add x a; lift.retval r` and
a checked structural `adjoint_table`/translation claim only, no comparison
claim, no asserted census and no bound schedule. Supply all ordinary
identity, signature, frame and claim fields as required. The following
JSON is the verification fragment under test, not a complete certificate
or a newly introduced certificate field:

<!-- structural-only-oracle -->
```json
{"kind":"oracle","verification":{"oracle":null,"fault_rate":null}}
```

| Request for the same structurally valid packet | Expected result |
|---|---|
| Standalone structural validation, no census asserted | Accept as a structural packet. |
| Executable acceptance requiring a fault census | Reject that acceptance claim: null discharges no census obligation. |
| Scheduled acceptance requiring all oracle contexts, or a zero-fault gate | Reject that acceptance claim: no RATE/context coverage is supplied. |
| Remove fault_rate instead of using null | Reject schema validation: required key missing. |
| Assert a census while retaining null | Reject the asserted census. |

With the same body, args=["1"] and B=1, a verified RATE counts
256 code-1 faults over the full 4×4×16 domain; with B=2 it counts zero.
Those are actual census results and cannot be inferred from the null
packet. The focused script checks these applicability outcomes, action
null versus omitted keys, the fixture migration, unchanged owner rules
and the retained recount/bundle text. It is a finite contract test,
not an implementation of the proposed production certificate checker.

### Paragraph byte identity and verification

Extract exactly the UTF-8 bytes between the two `muse-fault-rate-paragraph`
markers, excluding the marker lines and the paragraph's terminating LF.
There is no line reflow or Unicode normalization. These are the paragraph
bytes quoted by Muse in 0jq-M1; the extra conditional oracle sentence
is copied verbatim from checker-fault-rate-M1 in both notes.

| Note | Shared paragraph SHA-256 |
|---|---|
| 0jq.md | `b34376b04d0d2e295018245c1f7f3ce36b4f55ad5ed18e8f127119e36b9685b0` |
| checker-fault-rate.md | `b34376b04d0d2e295018245c1f7f3ce36b4f55ad5ed18e8f127119e36b9685b0` |

Verification: `timeout 30s python3 -I -B /home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-notefix-s4/check_notes.py joint`
exited 0. Both extracted hashes equal the values above and Muse's exact
paragraph. No source, fixture, spec or catalog was edited.

Every producer and closed-schema reader using these amended 0.9
certificate bytes must upgrade together for the required nullable field
and context/evidence fields; older readers cannot silently accept them.
Remaining council work: joint sol+astra review, then a Muse acceptance
seat. D2 keeps 0jq HELD until those conditions pass. Publication and
producer/checker implementation follow acceptance; this revision performs
none of those steps and reserves no K number.
