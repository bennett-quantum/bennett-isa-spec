#!/usr/bin/env python3
"""Canonical K83 source/coordinate vectors; hash/check via the published CLI."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
REF_HASH = "5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197"

def j(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)

def sig(mut=(), ins=(), bounds=()):
    return dict(abi="0.9", args=[], bounds=list(bounds), **{"in": list(ins)}, kind="perm_action", mem=[], mut=list(mut), tier="q")

def document(functions, data, binary, temporary):
    symbols = {}
    for name, signature, body in functions:
        symbols[str(len(symbols))] = dict(name=name, kind="perm_action", signature=signature, interface_hash="0" * 64)
    for name, image in data:
        symbols[str(len(symbols))] = dict(name=name, kind="data", signature=dict(abi="0.9", kind="data", size=len(image)//2, tier="-"), interface_hash="0"*64)
    def render():
        text = "bennett-object " + " ".join(f"{name}=0.9" for name in ("isa", "object", "launch", "sched", "cert", "coverage", "target")) + " ptr_width=32\n\nsection symbols\n" + j(symbols) + "\n\nsection functions\n"
        text += "\n".join(body for _, _, body in functions) + "\n"
        if data:
            text += "\nsection data\n" + "".join(f"{name} {image}\n" for name, image in data)
        return text
    temporary.write_text(render())
    hashes = json.loads(subprocess.run([binary, "hash", str(temporary)], check=True, capture_output=True, text=True).stdout)["symbols"]
    for symbol in symbols.values():
        symbol["interface_hash"] = hashes[symbol["name"]]["interface_hash"]
    text = render()
    temporary.write_text(text)
    subprocess.run([binary, "check", str(temporary)], check=True, capture_output=True, text=True)
    return text

def event(gate, point, ordinal, owner, bit=None, edge=None):
    return [gate, point, ordinal, owner, bit, [], [], edge]

def image_events(image, owner="a0", edge=None):
    bits = [bit for bit in range(8) if int(image, 16) >> bit & 1]
    return ([event("alloc", "$construct", 0, owner, edge=edge)]
            + [event("x", "$construct", i+1, owner, bit, edge) for i, bit in enumerate(bits)]
            + [event("x", "$release", i, owner, bit, edge) for i, bit in enumerate(reversed(bits))]
            + [event("release", "$release", len(bits), owner, edge=edge)])

def generate(binary, temporary):
    files = {}
    cases = []
    def add(name, entry, functions, data, bundles, actuals, expected, full, widths):
        text = document(functions, data, binary, temporary)
        filename = name + ".bisa"
        files[filename] = text
        cases.append(dict(name=name, object=filename, sha256=hashlib.sha256(text.encode()).hexdigest(), entry=entry, bundles=bundles, actuals=actuals, trace=expected, complete_trace=full, allocation_widths=widths))
    zero = "perm_action @gap (t:wire) () () () ()\nregs {  } bounds {0}\n^entry():\nqloop 0 {\nqregion local 0 1\n}\nx t"
    add("zero-trip", "gap", [("gap", sig(mut=[1], bounds=[0]), zero)], [], [["t",1]], ["t"], [event("alloc","$construct",0,"a0"),event("x","entry:2",0,"t",0),event("release","$release",0,"a0")], True, {"a0":1})
    arms = "perm_action @arms (t:wire) (c:wire in) () () ()\nregs {  } bounds {0, 0}\n^entry():\nqif c {\nqloop 0 {\nqregion left 0 1\n}\n} {\nqloop 0 {\nqregion right 0 1\n}\n}\nx t"
    add("qif-arms", "arms", [("arms",sig(mut=[1],ins=[1],bounds=[0,0]),arms)], [], [["t",1],["c",1]], ["t","c"], [event("alloc","$construct",0,"a0"),event("alloc","$construct",1,"a1"),event("release","$release",0,"a1"),event("release","$release",1,"a0")], False, {"a0":1,"a1":1})
    child = "perm_action @image () () () () ()\n^entry():\nqregion r 0 8 @blob"
    for image in ["01","81"]:
        add("image-"+image,"image",[("image",sig(),child)],[("blob",image)],[],[],image_events(image),True,{"a0":8})
    controlled = "perm_action @controlled () (c:wire in) () () ()\n^entry():\nctl c action_call @image () () () ()"
    add("controlled-image-81","controlled",[("controlled",sig(ins=[1]),controlled),("image",sig(),child)],[("blob","81")],[["c",1]],["c"],image_events("81",edge=[]),True,{"a0":8})
    repeat = "perm_action @repeat () () () () ()\nregs {  } bounds {2}\n^entry():\nqloop 2 {\naction_call @image () () () ()\n}"
    add("repeated-image-81","repeat",[("repeat",sig(bounds=[2]),repeat),("image",sig(),child)],[("blob","81")],[],[],image_events("81",edge=[0])+image_events("81",owner="a1",edge=[1]),True,{"a0":8,"a1":8})
    files["k83.json"] = j(dict(format="bennett.k83.v1", isa="0.9", ref_hash=REF_HASH, cases=cases)) + "\n"
    return files

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check",action="store_true")
    parser.add_argument("--ref",default=os.environ.get("BENNETT_CONFORMANCE_BIN",str(ROOT/"reference-interpreter/target/debug/bennett-isa-ref")))
    args = parser.parse_args()
    assert hashlib.sha256(b"isa=0.9\nexpansion=ref\nversion=0.9\n"+(ROOT/"spec/ref-0.9.json").read_bytes()+b"\n").hexdigest() == REF_HASH
    with tempfile.TemporaryDirectory(prefix="k83-",dir=os.environ["TMPDIR"]) as temporary:
        files = generate(args.ref,Path(temporary)/"object.bisa")
    for name, text in files.items():
        path = HERE/name
        if args.check:
            assert path.read_text() == text, f"stale vector: {path}"
        else:
            path.write_text(text)
    print(f"K83: {len(files)-1} canonical source vectors checked")
if __name__ == "__main__":
    main()
