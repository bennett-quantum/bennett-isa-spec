#!/usr/bin/env python3
"""Hand-derived K86 model obligations, distinct from certificate verdicts.

Only the published controlled_U encoding is used. No Kraus/instrument or
preparation attachment production is invented. All JSON numbers here are
integers, so stdlib sorted compact JSON is RFC 8785 for these ASCII documents.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "conformance/fixtures/accepted-k86-09"


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def digest(data):
    return hashlib.sha256(data).hexdigest()


def model(sign=1, own=0, total=1):
    zero, unit = ["0", "1"], [str(sign), "1"]
    inherited = total-own
    return dict(version="bennett.native-model/1", kind="unitary", arity=1,
                basis=["t"], parameter_bindings={"k": "k", "c": "c"},
                control_models={"kind": "controlled_U", "controls": "c", "disabled": "identity"},
                operand_map=list(range(inherited, total))+list(range(inherited))+[total],
                data={"matrix": [[[zero, zero], [unit, zero]], [[unit, zero], [zero, zero]]]})


def generate():
    original = ROOT / "conformance/fixtures/controlled-action-09"
    template = json.loads((original / "target.json").read_bytes())
    schedule_template = json.loads((original / "schedule.json").read_bytes())
    # Preserve the existing source's exact text and published identities. Its
    # schedule/certificate are independent records, not source identity bytes.
    source = (original / "controlled.bisa").read_bytes().split(b"section schedule\n")[0].rstrip()+b"\n"
    files = {"source.bisa": source}
    cases = []

    def add(name, supplied, epsilon=0, own=0, total=1, outcome="checked", theta=None, cost=None):
        target = copy.deepcopy(template)
        entry = target["gates"][0]
        entry.update(op="cx" if own == 1 else "x", controls={"min": 0, "max": 2}, epsilon=epsilon)
        if supplied is not None:
            entry["realisation_model"] = copy.deepcopy(supplied)
        if cost is not None:
            entry["cost"] = cost
        if theta is not None:
            entry.update(op="rz", params=["theta"])
        data = canonical(target)
        filename = name+".target.json"
        files[filename] = data
        context = dict(own_controls=own, total_controls=total, requested_epsilon=["3", "1"])
        if theta is not None:
            context["theta_bits"] = theta
        cases.append(dict(name=name, target=filename, target_hash=digest(data), context=context, outcome=outcome))
        return target

    exact = add("controlled-x", model())
    add("controlled-cx", model(own=1, total=2), own=1, total=2)
    add("negative-x-exact", model(-1), outcome="reject")
    add("negative-x-bound-one", model(-1), epsilon=1, outcome="reject")
    add("negative-x-bound-two", model(-1), epsilon=2)
    add("negative-x-bound-three", model(-1), epsilon=3)
    add("not-normalized", model(2), epsilon=3, outcome="reject")
    add("missing-positive", None, epsilon=1, outcome="unchecked")
    add("omitted-expression-zero", None, epsilon={"op": "-", "args": [1, 1]})
    add("context-zero", None, epsilon={"var": "c"}, total=0)
    add("context-positive", None, epsilon={"var": "c"}, outcome="unchecked")
    unknown = model()
    unknown["version"] = "bennett.native-model/99"
    add("unsupported-version", unknown, outcome="unchecked")
    bad_map = model(own=1, total=2)
    bad_map["operand_map"] = [0, 1, 2]
    add("wrong-control-order", bad_map, own=1, total=2, outcome="reject")
    bad_map["operand_map"] = [1, 1, 2]
    add("duplicate-operand", bad_map, own=1, total=2, outcome="reject")
    bad_leaf = model()
    bad_leaf["data"]["matrix"][0][1][0] = ["2", "2"]
    add("unreduced-leaf", bad_leaf, outcome="reject")
    add("zero-illegal-cost", None, epsilon={"op": "-", "args": [1, 1]},
        cost={"time": {"var": "eps"}}, outcome="reject")
    # The matrix is an exact identity and cannot establish a nonzero Rz ideal
    # identity without the transcendental checker. This is explicitly unchecked.
    angle = model()
    angle["parameter_bindings"]["theta"] = "theta"
    z, one = ["0", "1"], ["1", "1"]
    angle["data"]["matrix"] = [[[one, z], [z, z]], [[z, z], [one, z]]]
    add("binary64-angle", angle, epsilon=1, theta="3fb999999999999a", outcome="unchecked")

    # A complete, bound, admitted exact-X schedule. Adding model bytes changes
    # target and schedule hashes while the source identity is unchanged.
    schedule = copy.deepcopy(schedule_template)
    schedule["bindings"]["target_hash"] = digest(canonical(exact))
    files["controlled-x.schedule.json"] = canonical(schedule)
    legacy = copy.deepcopy(exact)
    del legacy["gates"][0]["realisation_model"]
    files["legacy.target.json"] = canonical(legacy)
    schedule["bindings"]["target_hash"] = digest(canonical(legacy))
    files["legacy.schedule.json"] = canonical(schedule)
    expected = {
        "controlled_x": [[1,0,0,0],[0,1,0,0],[0,0,0,1],[0,0,1,0]],
        "controlled_negative_x": [[1,0,0,0],[0,1,0,0],[0,0,0,-1],[0,0,-1,0]],
        "operator_norm_error": ["2", "1"],
        "numerical_frobenius_squared": ["8", "1"],
        "numerical_claims": [{"bound": 2, "accept": False}, {"bound": 3, "accept": True}],
        "self_numerical_bound": 0,
        "zero_channel_dual": {"Y_diagonal": ["1","8"], "mu": ["3","8"], "strict_pivot": ["1","8"]},
        "binary64_angle": ["3602879701896397", "36028797018963968"],
    }
    manifest = {"format": "bennett.native-model-vectors/1", "cases": cases, "expected": expected,
                "scope": "per-entry model obligations and independent process bounds; not executable certificate coverage",
                "files": {name: digest(data) for name, data in files.items()},
                "source_identity": {"object_hash": "45bbb506541e3a8bb28a18e1bb0aef16ba01ee1e7cac74d5a190deb3ad36ef5a",
                                    "controlled_interface_hash": schedule_template["bindings"]["closure_hash"]}}
    files["vectors.json"] = canonical(manifest)
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    files = generate()
    if args.check:
        for name, data in files.items():
            path = OUT / name
            if not path.is_file() or path.read_bytes() != data:
                raise SystemExit(f"stale vector: {path}")
        print(f"PASS: {len(files)} exact portable K86 files")
    else:
        OUT.mkdir(parents=True, exist_ok=True)
        for name, data in files.items():
            (OUT / name).write_bytes(data)


if __name__ == "__main__":
    main()
