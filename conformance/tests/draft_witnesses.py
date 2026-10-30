"""Check printed draft witnesses independently, without importing any implementation."""
from pathlib import Path
from fractions import Fraction
import hashlib
import json

root = Path(__file__).resolve().parents[2]
draft = (root / "spec/Bennett-ISA-Spec-0.9-draft.md").read_text()
accepted = root / "spec/Bennett-ISA-Spec.md"
assert hashlib.sha256(accepted.read_bytes()).hexdigest() == "90e61d9c377c939fe9652758f61cd5ead068ef06e4e40d812d3f3c5197ab605e"

def witness(name):
    marker = "**" + name + "**\n\n```json\n"
    assert marker in draft, "missing draft witness " + name
    return json.loads(draft.split(marker, 1)[1].split("\n```", 1)[0])

# Index is ceil(log10(1/eps)), evaluated with exact rationals, no float log.
for row in witness("EPS-DECADE witness"):
    eps = Fraction(row["eps"])
    n = 0
    while eps * 10**n < 1:
        n += 1
    assert n == row["index"]

block = witness("BLOCK-IMAGE witness")
image = bytes(512) + bytes([255]) * 512
assert len(image) == block["length"]
for index, expected in block["sector_first_bytes"]:
    assert image[512 * index] == expected
assert block["present_sectors"] == [0, 1]
assert block["absent_sector"] == 2
assert block["partial_lengths_rejected"] == [1, 511, 513, 1023]
assert all(n % 512 != 0 for n in block["partial_lengths_rejected"])

lib = witness("ISA-REF-LIB witness")
assert lib["name"] == "ref" and lib["isa"] == lib["version"] == "0.9"
recipe = lib["recipes"][0]
ops = recipe["operations"]
assert [op["ordinal"] for op in ops] == list(range(len(ops)))
assert recipe["scratch"] == recipe["removable"] == []
for s in range(4):
    for t in range(4):
        wires = {f"s[{i}]": (s >> i) & 1 for i in range(2)}
        wires.update({f"t[{i}]": (t >> i) & 1 for i in range(2)})
        for op in ops:
            assert op["gate"] == "cx"
            control, target = op["wires"]
            wires[target] ^= wires[control]
        out = sum(wires[f"t[{i}]"] << i for i in range(2))
        assert out == t ^ s
        assert sum(wires[f"s[{i}]"] << i for i in range(2)) == s
        for op in reversed(ops):
            control, target = op["wires"]
            wires[target] ^= wires[control]
        assert sum(wires[f"t[{i}]"] << i for i in range(2)) == t
        assert sum(wires[f"s[{i}]"] << i for i in range(2)) == s
payload = json.dumps(lib, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
envelope = "isa=0.9\nexpansion=ref\nversion=0.9\n" + payload + "\n"
hash_value = hashlib.sha256(envelope.encode()).hexdigest()
assert "Witness envelope SHA-256: `" + hash_value + "`." in draft
assert "### 11.1 Reference-library contract (K11)" in draft
assert not (root / "ref").exists(), "publication is gated on accepted 0.9"
print("draft witnesses: eps boundaries, block sectors, 16 qxor tuples, ordinals/hash; unchanged 0.8")

# hsd/clw witnesses: exact scheduled aggregation and replay independent of the VM.
est = witness("LAUNCH-ESTIMATE witness")
from decimal import Decimal, localcontext, ROUND_CEILING
from urllib.parse import quote

def decimal_upper(q, digits):
    with localcontext() as ctx:
        ctx.prec = digits
        ctx.rounding = ROUND_CEILING
        d = Decimal(q.numerator) / Decimal(q.denominator)
    text = format(d, "f")
    return text.rstrip("0").rstrip(".") if "." in text else text

# Nonterminating and exact values, both sides of a decimal exponent boundary.
for row in est["decimal_checks"]:
    q = Fraction(row["rational"])
    text = decimal_upper(q, est["precision_digits"])
    assert text == row["decimal"]
    assert Fraction(text) >= q
    assert q == 0 or len(text.replace(".", "").lstrip("0").rstrip("0")) <= est["precision_digits"]

rows = {}
end = max(Fraction(e["end"]) for e in est["live"])
rows["time"] = decimal_upper(end, est["precision_digits"])
for cls in est["classes"]:
    events = [e for e in est["live"] if e["class"] == cls]
    points = sorted({Fraction(e[k]) for e in events for k in ("start", "end")})
    peak = max((sum(e["count"] for e in events if Fraction(e["start"]) <= t < Fraction(e["end"])) for t in points), default=0)
    occupancy = sum((Fraction(e["end"]) - Fraction(e["start"])) * e["count"] for e in events)
    suffix = "[" + quote(cls, safe="-._~") + "]"
    rows["peak_qubits" + suffix] = str(peak)
    rows["qubit_second" + suffix] = decimal_upper(occupancy, est["precision_digits"])
assert sorted(rows.items()) == [tuple(row) for row in est["rows"]]
assert est["binding"]["args"] == ["2"]
assert est["binding"]["budget"] == ["2", "0", "0", "0"]

replay = witness("CLAIM-REPLAY witness")
before = replay["row"]["before"]
x, target, fault = map(int, before["bundles"])
a = int(replay["invocation"]["args"][0])
budget = int(replay["invocation"]["budget"][0])
assert budget == 2
expected = dict(before, bundles=[str(x), str(target ^ ((x + a) % 4)), str(fault)])
assert expected == replay["row"]["after"]
assert expected != replay["wrong_row"]["after"]
assert replay["wrong_row"]["after"] == before  # a self-comparison would miss this error
assert replay["changed_arg_after"] == dict(before, bundles=[str(x), str(target ^ ((x + 1) % 4)), str(fault)])
assert replay["short_budget_after"] == dict(before, bundles=[str(x), str(target), str(fault ^ 1)])
assert replay["sample_count"] == 4 and replay["distinct_samples"] == 1
assert Fraction(replay["covered_fraction"]) == Fraction(1, 2**11)
assert replay["verdict"] == "evidence"
for case_id, bead in [("G-estimate-resolved-class-decimal", "b-isa-spec-c15"), ("D-sampled-relation-invocation", "b-isa-spec-9uy")]:
    suite = case_id[0]
    case = json.loads((root / "conformance/cases" / suite / case_id / "case.json").read_text())
    assert "unavailable pending 0.9" in case["unavailable"]["reason"]
    assert bead in case["unavailable"]["reason"]
    assert "command" not in case  # no invented 0.8 CLI
print("draft witnesses: exact rational class rows/rounding and full-tuple sampled replay; pending 0.9")

binding = witness("CLAIM-BINDING witness")
assert binding["signature"] == ["i128", "f64", "token"]
assert 2**64 < int(binding["args"][0]) < 2**128
assert int(binding["args"][1], 16) == 1 << 63
assert binding["args"][2] == ["2", "0", "0", "0"]
encoded = json.dumps(binding["args"], separators=(",", ":")).encode()
assert hashlib.sha256(encoded).hexdigest() == binding["params_hash"]
changed = list(binding["args"])
changed[1] = "0x0000000000000000"
assert hashlib.sha256(json.dumps(changed, separators=(",", ":")).encode()).hexdigest() != binding["params_hash"]
# Carry at a decade boundary and >34-digit integer rounding still enclose.
assert decimal_upper(Fraction(10**35-1, 10**34), 34) == "10"
assert decimal_upper(Fraction(10**34+1), 34) == str(10**34+10)
