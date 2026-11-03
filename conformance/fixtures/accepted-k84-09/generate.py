#!/usr/bin/env python3
"""Publish K84 admission witnesses, hashing their standalone source via the CLI."""
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


def generate(binary, temporary):
    signature = dict(abi="0.9", args=[], bounds=[], **{"in": []},
                     kind="perm_action", mem=[], mut=[1], tier="q")
    symbol = dict(name="flip", kind="perm_action", signature=signature,
                  interface_hash="0" * 64)

    def source():
        envelope = " ".join(f"{key}=0.9" for key in
                            ("isa", "object", "launch", "sched", "cert", "coverage", "target"))
        return ("bennett-object " + envelope + " ptr_width=32\n\nsection symbols\n"
                + canonical({"0": symbol})
                + "\n\nsection functions\nperm_action @flip (t:wire mut) () () () ()\n"
                + "^entry():\nx t\n")

    def hashes(text):
        temporary.write_text(text)
        return json.loads(subprocess.run([binary, "hash", str(temporary)], check=True,
                                         capture_output=True, text=True).stdout)

    symbol["interface_hash"] = hashes(source())["symbols"]["flip"]["interface_hash"]
    base = source()
    identity = hashes(base)
    closure = identity["symbols"]["flip"]["closure_hash"]
    invocation = dict(args=[], budget=["0"] * 4)

    def tuple_(value):
        return dict(bundles=[str(value)], borrowed=[], regions=[])

    domain = dict(bundles=[dict(name="t", width=1, lo=0, hi=1)], borrowed=[], regions=[])
    claim = dict(kind="exhaustive", domain=domain, invocation=invocation,
                 subject=dict(closure_hash=closure, variant="forward", inherited_controls=0),
                 expected=dict(table=[dict(before=tuple_(x), after=tuple_(x ^ 1))
                                      for x in range(2)]))
    record = dict(kind="perm_action", signature=signature, bounds=[], claims=[claim],
                  cleanup={}, definedness_assumptions=[], gates_used=dict(x=dict(count=1)),
                  resources=dict(peak_qubits=1, qubit_second=1, time=1),
                  error={"forward:0": dict(inclusive=dict(operator_norm=0),
                                           local=dict(operator_norm=0))},
                  frame=dict(slots=[dict(id="mut:0", role="mut", width=1,
                                        **{"class": "fast"})],
                             scratch=[], history=[], borrow=[], ancillae=[]),
                  verification=dict(unitary=[0], inverse_pairs=[], borrow_frames=[],
                                    channel=[], oracle=None,
                                    fault_rate=dict(domain=dict(bundles=[], borrowed=[], regions=[]),
                                                    faults_by_code={}, undefined_excluded=0,
                                                    layout_dependent_excluded=0)))
    certificate = dict(
        versions={key: "0.9" for key in
                  ("isa", "object", "launch", "sched", "cert", "coverage", "target")},
        bindings=dict(object_hash=identity["object_hash"],
                      interface_hash=symbol["interface_hash"], closure_hash=closure,
                      reference_program_hash=closure, dialect_mapping_version="k84-vectors.1",
                      target_hash="0" * 64, invocation=invocation),
        producer=dict(name="conformance", version="1",
                      materialisation=dict(llvm_version="none", normaliser_version="1",
                                           pipeline="k84-vectors")),
        composition=dict(method="sum_over_multiplicities",
                         multiplicities={closure + ":forward:0": 1},
                         epsilon_total=dict(diamond=0)),
        regions={closure: record})
    rows = []
    files = {}
    for name, accepted, clause, obligation in [
        ("baseline", True, "§12.1 K13", "Standalone exhaustive NOT table with both input tuples."),
        ("missing-expected", False, "§12.1 K13", "The exhaustive claim must declare expected."),
        ("duplicate-component", False, "§12.2 K72", "One resolved interface slot occurs twice in domain.bundles."),
    ]:
        cert = copy.deepcopy(certificate)
        mutated = cert["regions"][closure]["claims"][0]
        if name == "missing-expected":
            del mutated["expected"]
        elif name == "duplicate-component":
            mutated["domain"]["bundles"].append(copy.deepcopy(domain["bundles"][0]))
        text = base + "\nsection certificate\n" + canonical(cert) + "\n"
        filename = name + ".bisa"
        files[filename] = text
        temporary.write_text(text)
        result = subprocess.run([binary, "check", str(temporary)], capture_output=True, text=True)
        assert result.returncode == (0 if accepted else 1), (name, result.stdout, result.stderr)
        if not accepted:
            assert not result.stdout, (name, result.stdout)
            assert clause.split()[0] in result.stderr, (name, result.stderr)
        rows.append(dict(name=name, object=filename, accepted=accepted,
                         sha256=hashlib.sha256(text.encode()).hexdigest(),
                         normative_clause=clause, obligation=obligation))
        case = dict(id="D-k84-" + name, suite="D", spec="ISA0.9 §12.3 K84; " + clause,
                    derivation=obligation + " Only the named certificate mutation differs from baseline.",
                    object="../../../fixtures/accepted-k84-09/" + filename,
                    command=["check", "{object}"],
                    expect=dict(exit=0, stdout_json=dict(status="ok")) if accepted else
                    dict(exit=1, stdout_empty=True, stderr_contains=[clause.split()[0]]))
        files["../../cases/D/" + case["id"] + "/case.json"] = json.dumps(
            case, ensure_ascii=False, indent=2) + "\n"
    files["k84.json"] = canonical(dict(format="bennett.k84.v1", isa="0.9", cases=rows)) + "\n"
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--ref", default=os.environ.get("BENNETT_CONFORMANCE_BIN", str(
        ROOT / "reference-interpreter/target/debug/bennett-isa-ref")))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="k84-", dir=os.environ["TMPDIR"]) as directory:
        files = generate(args.ref, Path(directory) / "object.bisa")
    for name, text in files.items():
        path = HERE / name
        if args.check:
            assert path.read_text() == text, f"stale vector: {path}"
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
    print("K84: standalone baseline and two negative certificate vectors checked")


if __name__ == "__main__":
    main()
