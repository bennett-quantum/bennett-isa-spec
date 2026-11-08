#!/usr/bin/env python3
"""Generate K93 vectors from §4.4/§15 formulas; CLI used ONLY for identities."""
import argparse
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
HEADER = "bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=8"


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def source(mutables, inputs, body):
    signature = {"abi": "0.9", "args": [], "bounds": [], "in": [w for _, w in inputs],
                 "kind": "action", "mem": [], "mut": [w for _, w in mutables], "tier": "q"}
    symbol = {"0": {"interface_hash": "0" * 64, "kind": "action", "name": "a", "signature": signature}}

    def params(values, role):
        return ", ".join(f"{n}:wire{f'[{w}]' if w != 1 else ''} {role}" for n, w in values)

    return (f"{HEADER}\n\nsection symbols\n{canonical(symbol)}\n\nsection functions\n"
            f"action @a ({params(mutables, 'mut')}) ({params(inputs, 'in')}) () () ()\n^entry():\n{body}\n")


def bits(values, widths):
    return "".join(str(v >> i & 1) for v, w in zip(values, widths) for i in range(w))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--ref", type=Path, help="published CLI, used for source identities only")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    objects = {}
    vectors = []
    cases = []

    def case(suite, name, obj, command, expect, derivation):
        ident = f"{suite}-k93-{name}"
        value = {"id": ident, "suite": suite, "spec": "ISA 0.9 §4.4 K93; §15 MCX operand and phase witnesses",
                 "derivation": derivation, "object": f"../../../fixtures/accepted-k93-09/{obj}.bisa",
                 "command": command, "expect": expect}
        cases.append((suite, ident, value))

    def relation(name, mutables, inputs, body, rows, derivation):
        objects[name] = source(mutables, inputs, body)
        widths = [w for _, w in mutables + inputs]
        count = len(mutables)
        vectors.append({"name": name, "object": f"{name}.bisa", "widths": widths,
                        "names": [n for n, _ in mutables + inputs], "mutables": count,
                        "derivation": derivation, "rows": rows})
        for ordinal, row in enumerate(rows):
            for adjoint in (False, True):
                before, after = (row["after"], row["before"]) if adjoint else (row["before"], row["after"])
                command = ["run-action", "{object}", "a", "--mut", ",".join(map(str, before[:count]))]
                if inputs:
                    command += ["--in", ",".join(map(str, before[count:]))]
                if adjoint:
                    command += ["--adjoint"]
                expect = {"exit": 0, "stdout_json": {"mut": after[:count], "in": after[count:],
                          "state": {bits(after, widths): [1, 0]}}, "stdout_json_exact_at": ["state"]}
                case("C", f"{name}-{ordinal}-{'adj' if adjoint else 'fwd'}", name, command, expect, derivation)

    derivation = "§15: SWAP first gives (q,p,c); MCX then XORs [c=3] into the first wire, with amplitude +1. The adjoint undoes MCX before SWAP."
    rows = [{"before": [p, q, c], "after": [q ^ (c == 3), p, c], "amplitude": [1, 0]}
            for p in range(2) for q in range(2) for c in range(4)]
    relation("witness", [("p", 1), ("q", 1)], [("c", 2)], "swap p q\nmcx (c) p", rows, derivation)
    for width in (1, 2, 3):
        rows = [{"before": [t, c], "after": [t ^ (c == (1 << width) - 1), c], "amplitude": [1, 0]}
                for t in range(2) for c in range(1 << width)]
        relation(f"width-{width}", [("t", 1)], [("c", width)], "mcx (c) t", rows,
                 "§4.4: AND of all increasing-index control bits equals [c=2^width-1]; target XORs it, exact phase +1.")
    rows = [{"before": [t, s, b], "after": [t ^ (s == 1 and b == 3), s, b], "amplitude": [1, 0]}
            for t in range(2) for s in range(2) for b in range(4)]
    for order in ("s, b", "b, s"):
        relation("mixed-" + ("sb" if order == "s, b" else "bs"), [("t", 1)], [("s", 1), ("b", 2)],
                 f"mcx ({order}) t", rows, "§15: own controls [s,b0,b1] or [b0,b1,s], k=3; t XOR (s AND [b=3]), exact phase +1.")
    rows = [{"before": [t, r, s, b], "after": [t ^ (r == s == 1 and b == 3), r, s, b], "amplitude": [1, 0]}
            for t in range(2) for r in range(2) for s in range(2) for b in range(4)]
    relation("predicate", [("t", 1)], [("r", 1), ("s", 1), ("b", 2)], "qpred r\nmcx (s, b) t\nqunpred r", rows,
             "§15: three explicit controls plus active r give c=4; toggle iff r=s=1 and b=3, phase +1.")
    for active in (False, True):
        rows = [{"before": [t, r], "after": [t ^ (r if active else 1), r], "amplitude": [1, 0]}
                for t in range(2) for r in range(2)]
        relation("empty-predicate" if active else "empty", [("t", 1)], [("r", 1)],
                 "qpred r\nmcx () t\nqunpred r" if active else "mcx () t", rows,
                 "§4.4: empty conjunction is true, k=0; X is still controlled by active r when present, exact phase +1.")
    for indexed in (False, True):
        width = 2 if indexed else 1
        rows = [{"before": [t, c], "after": [t ^ ((2 if indexed else 1) if c == 3 else 0), c], "amplitude": [1, 0]}
                for t in range(1 << width) for c in range(4)]
        name = "indexed-target" if indexed else "wire1-target"
        relation(name, [("t", width)], [("c", 2)], "mcx (c) t[1]" if indexed else "mcx (c) t", rows,
                 "§4.4: target is exactly one live wire; a bare wire[1] or t[1] is scalar; only that bit toggles for c=3.")
        if not indexed:
            objects[name] = objects[name].replace("t:wire mut", "t:wire[1] mut")
    negatives = [
        ("repeated-bundle", [("t", 1)], [("c", 2)], "mcx (c, c) t"),
        ("repeated-scalar", [("t", 1)], [("c", 1)], "mcx (c, c) t"),
        ("target-overlap", [("t", 2)], [], "mcx (t) t[0]"),
        ("scalar-target-overlap", [("t", 1)], [], "mcx (t) t"),
        ("multiwire-target", [("t", 2)], [("c", 2)], "mcx (c) t"),
        ("dead-target", [("t", 1)], [("c", 2)], "mcx (c) missing"),
        ("target-out-of-range", [("t", 2)], [("c", 2)], "mcx (c) t[2]"),
    ]
    for name, mutables, inputs, body in negatives:
        objects[name] = source(mutables, inputs, body)
        case("F", name, name, ["check", "{object}"],
             {"exit": 1, "stdout_empty": True, "stderr_contains": ["§3.3"]},
             "§4.4 applies §3.3 to every resolved control and the scalar live target before execution: repeated wires, overlap, missing/out-of-range or multiwire target reject as operands, independent of a relation claim.")
    for adjoint in (False, True):
        before, after = ("0000", "1011") if not adjoint else ("0000", "0011")
        # Components are unnormalised binary64 inputs; H-based reference test
        # separately checks exactly prepared 1/sqrt(2) amplitudes with external R.
        state = "0000=0.7071067811865476;0011=0.7071067811865476" if not adjoint else "0000=0.7071067811865476;1011=0.7071067811865476"
        case("C", f"coherent-{'adj' if adjoint else 'fwd'}", "witness",
             ["run-action", "{object}", "a", "--state", state] + (["--adjoint"] if adjoint else []),
             {"exit": 0, "stdout_json": {"state": {before: [0.7071067811865476, 0], after: [0.7071067811865476, 0]}},
              "stdout_json_exact_at": ["state"]},
             "§4.4/§15 linearity: exact +1 basis columns preserve both input binary64 amplitudes; the 00,c=3 branch toggles p. External-reference witness is separately recorded in k93.json and exercised by the reference test.")

    # Exact original witness bytes, including ptr_width=8 and its published hash.
    objects["witness"] = objects["witness"].replace("0" * 64, "e38d5da64f9a0ae89a66243bcbe5a321cb807a5bbabf83952b63d3465dfab346")
    outputs = {}
    for name, content in objects.items():
        destination = HERE / f"{name}.bisa"
        if args.ref and not any(name == n[0] for n in negatives):
            probe = HERE / "identity-input.bisa"
            probe.write_text(content)
            try:
                result = subprocess.run([str(args.ref.resolve()), "hash", str(probe)], check=True, capture_output=True, text=True)
            finally:
                probe.unlink()
            identity = json.loads(result.stdout)
            content = content.replace("0" * 64, identity["symbols"]["a"]["interface_hash"])
        elif destination.exists() and not any(name == n[0] for n in negatives):
            # Regeneration without CLI keeps already-published identity metadata.
            old = destination.read_text().split("section symbols\n")[1].splitlines()[0]
            content = content.replace("0" * 64, json.loads(old)["0"]["interface_hash"])
        outputs[destination] = content
    manifest = {"format": "bennett.k93.v1", "isa": "0.9", "vectors": vectors,
                "witness_object_hash": "db3d3f92a79077074901449d83532f1a0d82ef1621bd833adb7ad14d1aedd467",
                "witness_closure_hash": "e38d5da64f9a0ae89a66243bcbe5a321cb807a5bbabf83952b63d3465dfab346",
                "reference_witness": {"tuple_order": ["p", "q", "c", "R"], "before": [[0, 0, 0, 0], [0, 0, 3, 1]],
                                      "after": [[0, 0, 0, 0], [1, 0, 3, 1]], "amplitude": "1/sqrt(2)"},
                "conformance_cases": [ident for _, ident, _ in cases]}
    outputs[HERE / "k93.json"] = canonical(manifest) + "\n"
    for suite, ident, value in cases:
        outputs[ROOT / "conformance" / "cases" / suite / ident / "case.json"] = json.dumps(value, indent=2) + "\n"
    for path, content in outputs.items():
        if args.check:
            if path.read_text() != content:
                raise SystemExit(f"stale vector: {path}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
    print(f"K93: {len(vectors)} relations, {len(cases)} CLI vectors; expected values derived only from spec formulas")


if __name__ == "__main__":
    main()
