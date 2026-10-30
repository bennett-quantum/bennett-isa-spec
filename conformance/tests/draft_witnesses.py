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
assert "ISA-REF-LIB (from bennett-vm-47t): maintainer decision required" in draft
assert not (root / "ref").exists(), "publication is gated on accepted 0.9"
print("draft witnesses: eps boundaries, block sectors, 16 qxor tuples, ordinals/hash; unchanged 0.8")
