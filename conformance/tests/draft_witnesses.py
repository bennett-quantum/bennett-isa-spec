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
widths = replay["widths"]
assert widths == {"bundles": [2, 2, 4], "borrowed": [1], "regions": [[8, 8]]}
full_bits = sum(widths["bundles"] + widths["borrowed"] + sum(widths["regions"], []))
assert full_bits == 25
for pair in [replay["row"], replay["wrong_row"]]:
    for values in pair.values():
        for kind in ["bundles", "borrowed"]:
            assert all(0 <= int(v) < 2**w for v, w in zip(values[kind], widths[kind]))
        assert all(0 <= int(v) < 256 for cells in values["regions"] for v in cells)
assert Fraction(replay["covered_fraction"]) == Fraction(1, 2**full_bits)
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

# ASCII/integer-only subset: json.dumps is JCS for these fixtures.
grid = witness("TOPOLOGY witness")
grid_dir = root / "conformance/cases/E/E-topology-grid-09"
grid_bytes = json.dumps(grid, sort_keys=True, separators=(",", ":")).encode()
assert (grid_dir / "target.json").read_bytes() == grid_bytes
assert grid_bytes[-1:] == b"}"
assert grid["topology"]["vertices"] == [{"class": "fast"}] * 4
coords = [(x, y) for y in range(2) for x in range(2)]
edges = [[u, v] for u, p in enumerate(coords) for v, q in enumerate(coords) if u < v and abs(p[0]-q[0])+abs(p[1]-q[1]) == 1]
assert grid["topology"]["edges"] == edges
assert [0, 1] in edges and [0, 3] not in edges
assert grid["classes"]["fast"]["capacity"] == len(coords)
grid_hash = hashlib.sha256(grid_bytes).hexdigest()
assert json.loads((grid_dir / "case.json").read_text())["expected_outcome"]["target_hash"] == grid_hash
assert f"Target SHA-256: `{grid_hash}`." in draft

# Existing 0.8 CLI agreement check: text-order blocks with shared scratch.
import subprocess
multi_dir = root / "conformance/cases/C/C-multiblock-09"
source = (multi_dir / "two-blocks.bisa").read_text()
symbols = json.loads(source.split("section symbols\n")[1].split("\n")[0])
body = source.split("section functions\n")[1].rstrip("\n")
envelope = "isa=0.8\nptr_width=32\n" + json.dumps(symbols["0"]["signature"], sort_keys=True, separators=(",", ":")) + "\n" + body + "\n"
assert hashlib.sha256(envelope.encode()).hexdigest() == symbols["0"]["interface_hash"]
for t in range(4):
    for p in range(4):
        first = t ^ 1
        expected = first ^ ((first & 1) << 1) ^ p
        for value, adjoint, output in [(t, False, expected), (expected, True, t)]:
            command = [str(root / "reference-interpreter/target/debug/bennett-isa-ref"), "run-action", str(multi_dir / "two-blocks.bisa"), "flip", "--mut", str(value), "--in", str(p)]
            if adjoint:
                command.append("--adjoint")
            result = subprocess.run(command, capture_output=True, text=True)
            assert result.returncode == 0, result.stderr
            state = json.loads(result.stdout)
            assert state["mut"] == [output] and state["in"] == [p]

frame = witness("FRAME witness")
frame_dir = root / "conformance/cases/F/F-frame-records-09"
frame_bytes = json.dumps(frame, sort_keys=True, separators=(",", ":")).encode()
assert frame_bytes == (frame_dir / "frame.json").read_bytes()
frame_hash = hashlib.sha256(frame_bytes).hexdigest()
assert f"Frame SHA-256: `{frame_hash}`." in draft
frame_case = json.loads((frame_dir / "case.json").read_text())
assert frame_case["expected_outcome"]["frame_hash"] == frame_hash
assert set(frame) == {"slots", "scratch", "history", "borrow", "ancillae", "live_ranges"}
def frame_width(expr, c):
    if type(expr) is int:
        assert expr >= 0
        return expr
    if "var" in expr:
        assert expr == {"var": "c"}
        return c
    assert set(expr) == {"op", "args"} and expr["op"] == "+"
    a, b = expr["args"]
    return frame_width(a, c) + frame_width(b, c)
identities = set()
for category in ("slots", "scratch", "history", "borrow", "ancillae"):
    keys = {"id", "class", "width"} | ({"role"} if category == "slots" else {"state"} if category == "scratch" else {"scheme", "state"} if category == "ancillae" else set())
    for record in frame[category]:
        assert set(record) == keys
        assert record["id"] not in identities
        identities.add(record["id"])
    assert sum(frame_width(r["width"], 2) for r in frame[category]) == frame_case["expected_outcome"][category]
assert [r["id"] for r in frame["slots"]] == ["mut:0", "in:0"]
ranges = frame["live_ranges"]
assert [(r["start"], r["end"]) for r in ranges] == [(0, 2), (2, 4)]
assert all(a["end"] <= b["start"] or b["end"] <= a["start"] for i, a in enumerate(ranges) for b in ranges[i+1:] if a["frame"] == b["frame"])

draws = witness("DRAW-ORDER witness")
bit_order = ["00", "01", "10", "11"]
def measure(draw):
    return next(bits for i, bits in enumerate(bit_order) if Fraction(i+1, 4) > Fraction(draw))
bits = [measure(d) for d in draws["draws"]]
values = [sum(int(v) << i for i, v in enumerate(b)) for b in bits]
assert bits == draws["outcomes_bits"] and values == draws["outcomes_values"]
assert values[::-1] == draws["reversed_draw_values"]
assert [bit_order.index(b) for b in bits] == draws["integer_order_values"]
draw_dir = root / "conformance/cases/A/A-measure-draw-order-09"
result = subprocess.run([str(root / "reference-interpreter/target/debug/bennett-isa-ref"), "run-program", str(draw_dir / "two-measures.bisa"), "twice", "--draws", ",".join(draws["draws"])], capture_output=True, text=True)
assert result.returncode == 0, result.stderr
measurements = json.loads(result.stdout)["measurements"]
assert [m["bits"] for m in measurements] == bits
assert [int(m["value"]) for m in measurements] == values

# Recompute both 0.8 CLI witness envelopes without implementation imports.
source = (draw_dir / "two-measures.bisa").read_text()
symbols = json.loads(source.split("section symbols\n")[1].split("\n")[0])
functions = source.split("section functions\n")[1].rstrip("\n")
prep, main = functions.split("\nfn cc @twice", 1)
main = "fn cc @twice" + main
jcs = lambda obj: json.dumps(obj, sort_keys=True, separators=(",", ":"))
prep_envelope = "isa=0.8\nptr_width=32\n" + jcs(symbols["0"]["signature"]) + "\n" + prep + "\n"
prep_hash = hashlib.sha256(prep_envelope.encode()).hexdigest()
assert prep_hash == symbols["0"]["interface_hash"]
main_envelope = "isa=0.8\nptr_width=32\n" + jcs(symbols["1"]["signature"]) + "\n" + main.replace("@prepare", "@" + prep_hash) + "\n" + prep_hash + "\n"
assert hashlib.sha256(main_envelope.encode()).hexdigest() == symbols["1"]["interface_hash"]

# The ordered ratification inventory is closed, and the diff is byte-exact.
import re
assert "maintainer decision required" not in draft
assert not re.search(r"(?i)\bpropos(?:al|ed|es)\b", draft)
for k in range(9, 19):
    assert len(re.findall(rf"^\| K{k} \|", draft, re.M)) == 1
assert "target 0.9 has no topology field" not in draft
assert "regions.<closure_hash>.invocation" not in draft
assert "spec/ref-0.9-draft.json" in draft
for name, bead in [("E-ref-09-qxor-witness", "b-isa-spec-8r4"), ("E-topology-grid-09", "b-isa-spec-wr7"), ("C-multiblock-09", "b-isa-spec-2c9"), ("F-frame-records-09", "b-isa-spec-2c9"), ("A-measure-draw-order-09", "b-isa-spec-bas"), ("F-ratified-confirmations-09", "b-isa-spec-2c9")]:
    case = json.loads((root / "conformance/cases" / name[0] / name / "case.json").read_text())
    assert "unavailable pending 0.9" in case["unavailable"]["reason"]
    assert bead in case["unavailable"]["reason"]
result = subprocess.run(["diff", "-u", "--label", "a/spec/Bennett-ISA-Spec.md", "--label", "b/spec/Bennett-ISA-Spec-0.9-draft.md", str(accepted), str(root / "spec/Bennett-ISA-Spec-0.9-draft.md")], capture_output=True)  # stable labels: no absolute paths or mtimes in the committed diff
assert result.returncode == 1
assert result.stdout == (root / "spec/isa-0.8-to-0.9.diff").read_bytes()
