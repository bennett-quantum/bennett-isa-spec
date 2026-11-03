# K91 token equality witnesses

`token-aliases.bisa` contains a memoryless identity oracle, a two-call action,
an oracle with two token arguments, and the CC `compare` entry/R `sink` pair.
The tokens have tuple `(1,0,0,0)`; identities are supplied separately as canonical
partitions. No process-local ID is serialized.

`compare(a,b)` calls `sink(a)` (step 1), returns from the sink (step 2),
uncalls with `b` (step 3), then returns (step 4). `[0,0]` completes in four
steps. `[0,1]` faults at step 3 with code 4 and no results or effects. Both
complete prefixes reverse to the entry tuple. The oracle `context` prepends
its primary token; the five suite C requests cover every partition of three
equal tuples. Missing partitions and unequal tuples assigned the same label
reject as `E_ARG` before entry.

The action calls the identity oracle with `a`, then `b`, on unchanged x/t/f.
Both partitions produce identity on all 64 complete basis tuples. Only
`[0,0]` has a K81 matched-call pair, with sole K65 carrier
`pair/entry:1/iteration:[]/instance:[]/generated:0`. Duplicating its cleanup
selector rejects with K55's diagnostic. The `[0,1]` cleanup domain is empty,
so the same two selectors do not tie. Reference integration tests construct
the source circuit and check the published carrier, all 64 tuples, private
workspace cleanup and complete trace inverse without renumbering.

`pair-same.bisa` and `pair-distinct.bisa` include full-table unitary claims
bound to their respective partitions. Replay checks only these exhaustive
claims. Empty resource/gate maps assert no estimates or target-model counts;
these vectors do not establish target, resource, frame, or numerical claims.
Changing the root certificate to the other partition while keeping its
schedule rejects even though args and `params_hash` are identical.

`k91.json` pins every object byte hash, source/interface/closure/object
identities, params hash and schedule/certificate hashes. Partition-only
metadata changes preserve source identities while changing schedule and
certificate hashes. Suite F also covers first-label, gap, negative and arity
errors. These diagnostic checks require K91 attribution except for the
mandated K55 tie message; they do not register new normative diagnostic codes.

Run `python3 -I conformance/fixtures/accepted-k91-09/generate.py --check`
with `TMPDIR` set and optionally `--ref /path/to/bennett-isa-ref`. The generator
uses the CLI for hashing/admission and concrete CC/CLI outcomes. The expected
unitary is derived independently: identity(x) XORed into t twice leaves every
x, t and f unchanged. The tests do not rely on two interpreters agreeing.

Remaining K91 obligations, including inverse-pair coverage, ABI/loader
negotiation and lifecycle integration, remain under `b-isa-spec-8ui`.
