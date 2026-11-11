"""Independently check K12/K13/K91 publication vectors, not a launch estimate."""
from fractions import Fraction
import hashlib
import json
from pathlib import Path
from urllib.parse import quote


case_dir = Path(__file__).resolve().parent
root = case_dir.parents[3]
draft = (root / "spec/Bennett-ISA-Spec-0.9.md").read_text()


def witness(name):
    marker = "**" + name + "**\n\n```json\n"
    assert marker in draft, "missing publication witness: " + name
    return json.loads(draft.split(marker, 1)[1].split("\n```", 1)[0])


def jcs(value):
    # These vectors contain only ASCII strings, arrays and small integers.
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def digest(value):
    return hashlib.sha256(jcs(value)).hexdigest()


binding = witness("LAUNCH-PARAMETER-HASH witness")
assert binding == json.loads((case_dir / "binding.json").read_text())
case = json.loads((case_dir / "case.json").read_text())
args = binding["args"]
expected_hash = binding["params_hash"]
assert digest(args) == expected_hash == case["expected_outcome"]["params_hash"]
assert binding["token_aliases"] == [[0, 0, 0], [0, 1, 2]]
invocations = [
    dict(args=args, budget=binding["budget"], token_aliases=partition)
    for partition in binding["token_aliases"]
]
assert digest(invocations[0]) != digest(invocations[1])
assert all(digest(invocation) != expected_hash for invocation in invocations)
changed_budget = dict(invocations[0], budget=["3", "0", "0", "0"])
assert digest(changed_budget) != digest(invocations[0])
assert digest(changed_budget["args"]) == expected_hash
changed_args = [list(args[0]), list(args[1])]
changed_args[1][0] = "3"
assert digest(changed_args) != expected_hash
assert hashlib.sha256(jcs(args) + b"\n").hexdigest() != expected_hash
assert digest(["0x0000000000000000"]) != digest(["0x8000000000000000"])
assert binding["same_params_hash"] and not binding["same_invocation"]

# Recheck the existing class and decimal contract with exact arithmetic.
estimate = witness("LAUNCH-ESTIMATE witness")
rows = dict(estimate["rows"])
assert list(rows) == sorted(rows)
for resource in ["peak_qubits", "qubit_second"]:
    for name in estimate["classes"]:
        key = quote(resource, safe="-._~") + "[" + quote(name, safe="-._~") + "]"
        assert key in rows
        if name == "idle":
            assert rows[key] == "0"
assert quote("B]", safe="-._~") == "B%5D"
assert quote("B%5D", safe="-._~") == "B%255D"
for check in estimate["decimal_checks"]:
    value = Fraction(check["rational"])
    display = Fraction(check["decimal"])
    assert display >= value
    if value == 0:
        assert check["decimal"] == "0"
        continue
    exponent = 0
    while value < Fraction(10) ** exponent:
        exponent -= 1
    while value >= Fraction(10) ** (exponent + 1):
        exponent += 1
    quantum = Fraction(10) ** (exponent - 33)
    scaled = value / quantum
    rounded = -(-scaled.numerator // scaled.denominator) * quantum
    assert display == rounded
    text = check["decimal"]
    assert "e" not in text.lower() and not text.startswith("+")
    assert not ("." in text and text.endswith("0"))

print("PASS: args-only hash, separate alias/budget binding, class rows and exact upward decimals")
