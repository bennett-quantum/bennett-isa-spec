"""Reproduce the complete ref/0.9 catalog's independent-qjumpr-width repair."""
import argparse
import hashlib
import json
from pathlib import Path

here = Path(__file__).resolve().parent
base = (here / "ref-0.9.json").read_bytes()
envelope = b"isa=0.9\nexpansion=ref\nversion=0.9\n"
assert hashlib.sha256(envelope + base + b"\n").hexdigest() == "5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197"
document = json.loads(base)
recipe = next(r for r in document["recipes"] if r["kind"] == "qjumpr")


def address_width(value):
    if isinstance(value, dict):
        return {k: address_width(v) for k, v in value.items()}
    if isinstance(value, list):
        return list(map(address_width, value))
    if isinstance(value, str):
        return "A" if value == "P" else value.replace('"P"', '"A"')
    return value


recipe["body"] = address_width(recipe["body"])
copies = [node for node in recipe["body"] if node.get("for") == "bit"]
assert len(copies) == 1 and copies[0]["to"] == "A"
copies[0]["to"] = "CopyWidth"
parameters = recipe["parameters"]
assert parameters[0] == {"name": "P", "type": "integer"}
parameters[1:1] = [{"name": "A", "type": "integer"}, {"name": "CopyWidth", "type": "integer"}]
next(p for p in parameters if p["name"] == "a")["width"] = "A"
assert next(p for p in parameters if p["name"] == "succ")["width"] == "P"
recipe["source"] = "§3.3/§3.6/§4.2: qjumpr compares the full A-bit address against First/End; CopyWidth=min(A,P) low bits XOR into the P-bit successor; full-width flag and cause4 fault writes precede flagged successor selection."
# Catalog keys are ASCII; every numeric value is a small integer, so this
# serialization is the catalog's RFC 8785 subset, with no trailing LF.
raw = json.dumps(document, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()
parser = argparse.ArgumentParser()
parser.add_argument("--check", action="store_true")
args = parser.parse_args()
path = here / "ref-0.9-r1.json"
if args.check:
    assert path.read_bytes() == raw
else:
    path.write_bytes(raw)
print("ref/0.9 r1: 219 recipes; envelope SHA-256 " + hashlib.sha256(envelope + raw + b"\n").hexdigest())
