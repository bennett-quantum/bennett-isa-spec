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
    empty = "perm_action @empty (t:wire) () () () ()\n^entry():\nx t"
    files["empty-support.bisa"] = document([("empty",sig(mut=[1]),empty)],[],binary,temporary)
    schedule_cases = []
    def schedule_case(name, source, entry, entries, accepted, reason):
        text = files[source+".bisa"]
        temporary.write_text(text)
        identity = json.loads(subprocess.run([binary,"hash",str(temporary)],check=True,capture_output=True,text=True).stdout)["symbols"][entry]
        schedule = dict(bindings=dict(budget=[0,0,0,0],closure_hash=identity["closure_hash"],epsilon=0,inherited_controls=0,params_hash=hashlib.sha256(b"[]").hexdigest(),region=entry,scheduler=dict(name="portable-k83",version="1"),target_hash="0"*64,variant="forward"),entries=entries)
        full = text + "\nsection schedule\n" + j(schedule) + "\n"
        filename = "schedule-"+name+".bisa"
        files[filename] = full
        temporary.write_text(full)
        result = subprocess.run([binary,"check",str(temporary)],capture_output=True,text=True)
        assert result.returncode == (0 if accepted else 1), (name,result.stdout,result.stderr)
        if not accepted:
            assert "§11" in result.stderr, (name,result.stderr)
        schedule_cases.append(dict(name=name,object=filename,sha256=hashlib.sha256(full.encode()).hexdigest(),accepted=accepted,derivation=reason))
    def selector(source, entry, point, iteration=None, instance=None):
        symbols=json.loads(files[source+".bisa"].split("section symbols\n")[1].split("\n")[0])
        symbol=next(symbol["interface_hash"] for symbol in symbols.values() if symbol["name"]==entry)
        return dict(generated=0,kind="alloc",path=[dict(symbol=symbol,point=point,iteration=[] if iteration is None else iteration,instance=[] if instance is None else instance)],step="*",phase="*",alloc=dict(**{"class":"A"}))
    construct=selector("zero-trip","gap","$construct")
    image_construct=selector("image-81","image","$construct")
    release=selector("image-81","image","$release")
    release.pop("alloc");release["kind"]="gate";release["gate"]=dict(realisation="native",epsilon_share=0)
    schedule_case("valid-support","image-81","image",[image_construct,release],True,"Image81 alloc/0 and reverse-image X[7]/0 have legal action construct/release coordinates.")
    for point in ["$step","$commit"]:
        schedule_case("action-"+point[1:],"zero-trip","gap",[selector("zero-trip","gap",point)],False,"The reserved point "+point+" is oracle-only.")
    schedule_case("nonempty-iteration","zero-trip","gap",[selector("zero-trip","gap","$construct",iteration=[0])],False,"Hoisted action support has its own empty source-loop iteration vector even in a zero-trip declaration.")
    schedule_case("nonempty-instance","zero-trip","gap",[selector("zero-trip","gap","$construct",instance=[0])],False,"Action support has empty fixed-binding instance.")
    schedule_case("duplicate-support","zero-trip","gap",[construct,construct],False,"The mandatory function-entry allocation witnesses two maximal matching alloc entries, despite loop bound0.")
    no_support=selector("empty-support","empty","$construct")
    schedule_case("empty-support","empty-support","empty",[no_support,no_support],True,"An action with no local region has no construct support leaves and cannot witness a tie.")
    caller=selector("controlled-image-81","controlled","entry:0")["path"][0];caller["call_index"]=0
    callee=selector("controlled-image-81","image","$construct")
    callee["path"].insert(0,caller)
    schedule_case("controlled-duplicate-support","controlled-image-81","controlled",[callee,callee],False,"Callee region preparation is unconditional for both external control values, so duplicate support entries reject at admission.")
    files["k83.json"] = j(dict(format="bennett.k83.v1", isa="0.9", ref_hash=REF_HASH, cases=cases,schedule_cases=schedule_cases)) + "\n"
    for suite, rows in [("F",cases),("H",schedule_cases)]:
        for case in rows:
            accepted=case.get("accepted",True)
            item=dict(id=suite+"-k83-"+case["name"],suite=suite,spec="ISA0.9 §11 K83 Action-local region support; §15 suitesF/H",derivation=case.get("derivation","Canonical source admission for K83 "+case["name"]+"; exact source coordinates and reversible execution are gated by k83_source_support.rs."),object="../../../fixtures/accepted-k83-09/"+case["object"],command=["check","{object}"],expect=dict(exit=0,stdout_json=dict(status="ok")) if accepted else dict(exit=1,stdout_empty=True,stderr_contains=["§11"]))
            files["../../cases/"+suite+"/"+item["id"]+"/case.json"] = json.dumps(item,ensure_ascii=False,indent=2)+"\n"
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
            path.parent.mkdir(parents=True,exist_ok=True)
            path.write_text(text)
    print("K83: 6 source vectors and 8 schedule vectors checked")
if __name__ == "__main__":
    main()
