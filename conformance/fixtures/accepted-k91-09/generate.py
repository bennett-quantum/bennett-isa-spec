#!/usr/bin/env python3
"""Portable K91 partitions, matched-call carriers and invocation bindings."""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def sha(data):
    return hashlib.sha256(data.encode()).hexdigest()


def generate(binary, temporary):
    functions = [
        ("identity", dict(abi="0.9", args=[], fault=4, **{"in": [1]},
                          kind="oracle", mem=[], tgt=1, tier="q"),
         "oracle @identity (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x"),
        ("pair", dict(abi="0.9", args=["token", "token"], bounds=[], **{"in": [1]},
                      kind="perm_action", mem=[], mut=[1, 4], tier="q"),
         "perm_action @pair (t:wire mut, f:wire[4] mut) (x:wire in) () (a:token, b:token) ()\n"
         "^entry():\noracle_call @identity a (x) t f () ()\noracle_call @identity b (x) t f () ()"),
        ("context", dict(abi="0.9", args=["token", "token"], fault=4, **{"in": [1]},
                         kind="oracle", mem=[], tgt=1, tier="q"),
         "oracle @context (x:wire in) (t:wire) (f:wire[4]) () (a:token, b:token)\n^entry():\nlift.retval x"),
        ("sink", dict(abi="0.9", kind="r", params=["token"], results=[], tier="r"),
         "fn r @sink (a:token) () () () ()\n^entry():\nret"),
        ("compare", dict(abi="0.9", kind="cc", params=["token", "token"], results=[], tier="cc"),
         "fn cc @compare (a:token, b:token) () () () ()\n^entry():\ncall @sink a\nuncall @sink (b) ()\nret"),
    ]
    symbols = {str(i): dict(name=name, kind=sig["kind"], signature=sig,
                            interface_hash="0" * 64)
               for i, (name, sig, _) in enumerate(functions)}

    def source():
        envelope = " ".join(f"{name}=0.9" for name in
                            ("isa", "object", "launch", "sched", "cert", "coverage", "target"))
        return (f"bennett-object {envelope} ptr_width=32\n\nsection symbols\n"
                + canonical(symbols) + "\n\nsection functions\n"
                + "\n".join(body for _, _, body in functions) + "\n")

    def cli(text, *args, accepted=True):
        temporary.write_text(text)
        result = subprocess.run([binary, args[0], str(temporary), *args[1:]],
                                capture_output=True, text=True)
        assert result.returncode == (0 if accepted else 1), (args, result.stdout, result.stderr)
        if not accepted:
            assert not result.stdout, (args, result.stdout)
            return result.stderr
        return json.loads(result.stdout)

    identity = cli(source(), "hash")
    for symbol in symbols.values():
        symbol["interface_hash"] = identity["symbols"][symbol["name"]]["interface_hash"]
    base = source()
    identity = cli(base, "hash")
    closure = identity["symbols"]["pair"]["closure_hash"]
    signature = functions[1][1]
    args = [["1", "0", "0", "0"]] * 2
    params_hash = sha(canonical(args))
    cleanup = dict(cleanup=dict(realisation="recompute"), generated=0, kind="cleanup",
                   path=[dict(symbol=closure, point="entry:1", iteration=[], instance=[])],
                   phase="*", step="*")
    domain = dict(bundles=[dict(name=n, width=w, lo=0, hi=(1 << w) - 1)
                           for n, w in [("t", 1), ("f", 4), ("x", 1)]],
                  borrowed=[], regions=[])
    table = []
    for t in range(2):
        for f in range(16):
            for x in range(2):
                state = dict(bundles=list(map(str, [t, f, x])), regions=[], borrowed=[])
                table.append(dict(before=state, after=copy.deepcopy(state)))
    files = {"token-aliases.bisa": base}
    schedules = []
    fixtures = []
    cases = []

    def add_case(suite, name, filename, command, expected, derivation):
        case = dict(id=f"{suite}-k91-{name}", suite=suite,
                    spec="ISA0.9 §§5.3,5.4,7.4,11,12.3 K91",
                    derivation=derivation, object=f"../../../fixtures/accepted-k91-09/{filename}",
                    command=command, expect=expected)
        files[f"../../cases/{suite}/{case['id']}/case.json"] = json.dumps(
            case, ensure_ascii=False, indent=2) + "\n"
        cases.append(case["id"])

    def add_object(name, text, accepted=True, clause="K91"):
        filename = name + ".bisa"
        result = cli(text, "check", accepted=accepted)
        if not accepted:
            assert clause in result, (name, result)
        files[filename] = text
        fixtures.append(dict(name=name, object=filename, sha256=sha(text), accepted=accepted))
        add_case("F", name, filename, ["check", "{object}"],
                 dict(exit=0, stdout_json=dict(status="ok")) if accepted else
                 dict(exit=1, stdout_empty=True, stderr_contains=[clause]),
                 "Only the named binding differs from the same standalone source; "
                 "canonical equality classes and equal-label tuple consistency are mandatory.")
        return filename

    certificates = []
    for name, labels in [("same", [0, 0]), ("distinct", [0, 1])]:
        invocation = dict(args=args, budget=["0"] * 4, token_aliases=labels)
        schedule = dict(bindings=dict(budget=[0] * 4, closure_hash=closure, epsilon=0,
                                     inherited_controls=0, params_hash=params_hash,
                                     region="pair", scheduler=dict(name="k91-vectors", version="1"),
                                     target_hash="0" * 64, token_aliases=labels, variant="forward"),
                        entries=[cleanup])
        claim = dict(kind="exhaustive", domain=domain, invocation=invocation,
                     subject=dict(closure_hash=closure, variant="forward", inherited_controls=0),
                     expected=dict(table=table))
        # Only the exhaustive unitary claim is asserted by replay tests. Empty
        # resource/gate maps supply no target-model estimates or gate counts.
        template = json.loads((HERE.parent / "accepted-k84-09/baseline.bisa").read_text()
                              .split("section certificate\n", 1)[1])
        record = copy.deepcopy(next(iter(template["regions"].values())))
        record.update(signature=signature, claims=[claim], resources={}, gates_used={},
                      frame=dict(slots=[dict(id=slot, role=role, width=w, **{"class": "any"})
                                        for slot, role, w in [("mut:0", "mut", 1),
                                                              ("mut:1", "mut", 4),
                                                              ("in:0", "in", 1)]],
                                 scratch=[], history=[], borrow=[], ancillae=[]))
        certificate = copy.deepcopy(template)
        certificate["bindings"].update(object_hash=identity["object_hash"],
                                       interface_hash=closure, closure_hash=closure,
                                       reference_program_hash=closure,
                                       dialect_mapping_version="k91-vectors.1",
                                       invocation=invocation, schedule_hash=sha(canonical(schedule)))
        certificate["producer"]["materialisation"]["pipeline"] = "k91-vectors"
        certificate["regions"] = {closure: record}
        certificate["composition"]["multiplicities"] = {closure + ":forward:0": 1}
        text = base + "\nsection schedule\n" + canonical(schedule) + "\n\nsection certificate\n" + canonical(certificate) + "\n"
        filename = add_object("pair-" + name, text)
        certificates.append((schedule, certificate))
        schedules.append(dict(name=name, token_aliases=labels, params_hash=params_hash,
                              schedule_hash=sha(canonical(schedule)), certificate_hash=sha(canonical(certificate)),
                              object=filename, claim=0, full_table_elements=64,
                              carriers=[dict(symbol=closure, point="entry:1", iteration=[],
                                             instance=[], generated=0)] if name == "same" else []))
        repeated = copy.deepcopy(schedule)
        repeated["entries"].append(copy.deepcopy(cleanup))
        add_object("duplicate-carrier-" + name,
                   base + "\nsection schedule\n" + canonical(repeated) + "\n",
                   accepted=name == "distinct", clause="multiple maximal matching schedule entries")
        command = ["run-cc", "{object}", "compare", "--arg", "token:1,0,0,0",
                   "--arg", "token:1,0,0,0", "--token-aliases", canonical(labels)]
        observed = cli(base, command[0], *command[2:])
        expected = dict(status="COMPLETED" if name == "same" else "FAILED",
                        fault=None if name == "same" else 4,
                        steps=4 if name == "same" else 3, results=[], effect_tape_length=0)
        for key, value in expected.items():
            assert observed[key] == value, (name, key, observed)
        add_case("G", "call-uncall-" + name, "token-aliases.bisa", command,
                 dict(exit=0, stdout_json=expected),
                 "call is step1, sink ret step2, uncall step3, root ret step4. "
                 "One actual identity matches the segment; two equal-tuple identities mismatch at step3/fault4.")
        reverse = cli(base, "reverse", *command[2:])
        assert reverse["equals_entry"] and reverse["fault"] == expected["fault"], reverse
        add_case("G", "reverse-" + name, "token-aliases.bisa", ["reverse", *command[1:]],
                 dict(exit=0, stdout_json=dict(equals_entry=True, fault=expected["fault"])),
                 "Reverse the full successful or failed call prefix to the complete entry state.")

    for suffix, labels in [("first-label", [1, 1]), ("gap", [0, 2]),
                           ("negative", [-1, 0]), ("arity", [0])]:
        schedule = copy.deepcopy(certificates[0][0])
        schedule["bindings"]["token_aliases"] = labels
        add_object("invalid-" + suffix, base + "\nsection schedule\n" + canonical(schedule) + "\n", False)
    schedule, certificate = copy.deepcopy(certificates[0])
    certificate["bindings"]["invocation"]["token_aliases"] = [0, 1]
    add_object("wrong-partition-certificate", base + "\nsection schedule\n" + canonical(schedule)
               + "\n\nsection certificate\n" + canonical(certificate) + "\n", False)
    for name, extra, second in [("missing", [], "1"), ("unequal-tuples", ["--token-aliases", "[0,0]"], "2")]:
        command = ["run-cc", "{object}", "compare", "--arg", "token:1,0,0,0",
                   "--arg", f"token:{second},0,0,0", *extra]
        assert "E_ARG" in cli(base, command[0], *command[2:], accepted=False)
        add_case("G", "invalid-request-" + name, "token-aliases.bisa", command,
                 dict(exit=1, stdout_empty=True, stderr_contains=["E_ARG"]),
                 "A tuple-only request must supply an arity-correct canonical partition; equal labels require equal tuples before entry.")
    for labels in [[0, 0, 0], [0, 0, 1], [0, 1, 0], [0, 1, 1], [0, 1, 2]]:
        command = ["run-oracle", "{object}", "context", "--input", "1", "--budget", "1,0,0,0",
                   "--arg", "token:1,0,0,0", "--arg", "token:1,0,0,0", "--token-aliases", canonical(labels)]
        observed = cli(base, command[0], *command[2:])
        assert observed == dict(payload=1, code=0, steps=1), observed
        add_case("C", "three-tokens-" + "".join(map(str, labels)), "token-aliases.bisa", command,
                 dict(exit=0, stdout_json=observed),
                 "The oracle prepends the primary token. These are all five consistent partitions of three equal tuples; identity returns x in one row.")
    command = ["run-oracle", "{object}", "identity", "--input", "1", "--budget", "1,0,0,0", "--token-aliases", "[0]"]
    assert cli(base, command[0], *command[2:]) == dict(payload=1, code=0, steps=1)
    add_case("C", "token-free-oracle", "token-aliases.bisa", command,
             dict(exit=0, stdout_json=dict(payload=1, code=0, steps=1)),
             "A token-free oracle has exactly one primary identity slot, [0].")
    files["k91.json"] = canonical(dict(format="bennett.k91.v1", isa="0.9",
                                      object_hash=identity["object_hash"], symbols=identity["symbols"],
                                      source=dict(object="token-aliases.bisa", sha256=sha(base)),
                                      params_hash=params_hash, partitions=schedules,
                                      fixtures=fixtures, conformance_cases=cases)) + "\n"
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--ref", default=os.environ.get("BENNETT_CONFORMANCE_BIN", str(
        ROOT / "reference-interpreter/target/debug/bennett-isa-ref")))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="k91-", dir=os.environ["TMPDIR"]) as directory:
        files = generate(args.ref, Path(directory) / "object.bisa")
    for name, text in files.items():
        path = HERE / name
        if args.check:
            assert path.read_text() == text, f"stale vector: {path}"
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
    print("K91: identity partitions, CC prefixes, source carriers and binding rejects checked")


if __name__ == "__main__":
    main()
