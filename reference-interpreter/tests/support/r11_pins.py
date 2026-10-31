"""Independent R11 literal envelopes. Standard Python hashlib/json only.
Malformed objects are retained as evidence, never normalized by the interpreter.
"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
def j(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"))
def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()
HEADER = "bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n"
cases = {}
def case(tag, funcs, layout=None, expected=None):
    symbols, hashes, envelopes = {}, {}, {}
    for name, sig, body, refs in funcs:
        envelope = "isa=0.8\nptr_width=32\n" + j(sig) + "\n" + body
        for ref in refs:
            envelope = envelope.replace("@"+ref+" ", "@"+hashes[ref]+" ")
        if layout is not None and sig["kind"] == "oracle":
            # Select structured owners before serializing dotted paths.
            leaves = {k:v for k,v in layout.items() if k not in ("globals","interface_regions")}
            leaves.update({"globals."+k:v for k,v in layout["globals"].items()})
            leaves.update({"interface_regions."+name+"."+k:v
                           for k,v in layout["interface_regions"].get(name,{}).items()})
            envelope += "".join("layout "+k+" "+j(v)+"\n" for k,v in sorted(leaves.items()))
        envelope += "".join(hashes[ref]+"\n" for ref in refs)
        hashes[name] = sha(envelope)
        envelopes[name] = envelope
        symbols[str(len(symbols))] = dict(name=name,kind=sig["kind"],signature=sig,interface_hash=hashes[name])
    text = HEADER+"\nsection symbols\n"+j(symbols)+"\n\nsection functions\n"+"".join(f[2] for f in funcs)
    if layout is not None:
        text += "\nsection layout\n"+j(layout)+"\n"
    if expected:
        for name, pin in expected[0].items():
            assert hashes[name] == pin, (tag,name,hashes[name],pin)
        if expected[1]:
            assert sha(text) == expected[1], (tag,sha(text),expected[1])
    cases[tag] = dict(text=text,hashes=hashes,envelopes=envelopes,object_hash=sha(text))
    print(tag, hashes, sha(text))

action = dict(abi="0.8",args=[],bounds=[],**{"in":[]},kind="action",mem=[],mut=[1],tier="q")
head = "action @a (t:wire mut) () () () ()\n"
case("wire", [("a",action,head+"regs { d:wire }\n^entry():\ncx d t\n",[])],
     expected=({"a":"4a592df0f0a8651417bf00a48c5b75ad86060cf4a3b5d157a8c3bbce5443969d"},"c90b514421ef25e2cd27cd1b80039a39ca9a5c8d78173968ac4e5c25f7eb988a"))
case("angle", [("a",action,head+"regs { angle:f64 }\n^entry():\nrx angle t\n",[])],
     expected=({"a":"544dfe0fb1b3e44d693ddb39967d5d5b043b9f92f4eb33e43b2d402a1d433ec6"},"918e9042f73b4f1f0cd72f9364e902aacc35e1cad3fcd8976ec9b0139cec2978"))
case("producer", [("a",action,head+"^entry():\nd = qalloc any 1\ncx d t\nqfree d\n",[])],
     expected=({"a":"ed0795bb5a01b03715111e2fefe4bedb59a6869b3bdaab84f80fe7bc0e51fe05"},"bef5f1ac1aa80ab0a4f523fe972f9227695b44f75e49c956acde273a9d7c225b"))
case("argument", [("a",dict(action,args=["f64"]),"action @a (t:wire mut) () () (angle:f64) ()\n^entry():\nrx angle t\n",[])],
     expected=({"a":"58e8b3a871bfc8167844e4175bbc7e6ef9cf17ccfbf1acd30b33800a49910c80"},"5d4101e0c48ea4074fd97cc9a70e869779518b2ff864ec9afbba548ebf48af45"))
def oracle(name, width, region):
    w = "wire" if width==1 else "wire[8]"
    mem = "(" + region + ":region[0,8] in)" if region else "()"
    sig = dict(abi="0.8",args=[],fault=4,**{"in":[width]},kind="oracle",
               mem=[dict(A=0,W=8,role="in")] if region else [],tgt=width,tier="q")
    return (name,sig,f"oracle @{name} (x:{w} in) (t:{w}) (f:wire[4]) {mem} ()\n^entry():\nlift.retval x\n",[])
for base, pin in [(2,"9c6ed8677516f307a9957360c5328bc6df33d2263c2af75978064a7cbdc64863"),(3,"e3e41f7b25615cb9b97efaeff328cddff88ec8028777fd7d0745d2675eff85d5")]:
    layout = dict(A_tab=0,H=4,S_cells=0,globals={},heap_base=1,heap_limit_max=0,
                  interface_regions={"a.b":{"db":{"base":base}}},null_size=1,ptr_width=32)
    case("sol-layout-"+str(base),[oracle("a",1,None),oracle("a.b",1,"db")],layout,
         ({"a":"1b593b9027243623bfb18cc5b34c51d7df81d23526bc4987c14a5da2806b9521","a.b":pin},
          "bccb8459c5f0746c49f98dad438d40a71e56d5e9533c8dc714bf32391c401877" if base==2 else None))
for base, pin, op in [(32,"26455a8ebeb96ca9ef90972709611ad6eed2bd2cb18f61aa368312e19edbd160","1292bb07226f33217b884470c8e2c486bd4fb8b0f1eb4d9c829aaa75597cb4f4"),
                      (48,"03ae1cd76644a677efd6819e42cf6ec74036c7aeeedbb431fffc89caa7aa441e","9f818c01a67c28872cb3e28e5a4c4405616cd0c0d3510576110ab7b8d90ba0ab")]:
    layout = dict(A_tab=1,H=8,S_cells=0,globals={},heap_base=128,heap_limit_max=0,
                  interface_regions={"a":{"db":{"base":16}},"a.b":{"other":{"base":base}}},
                  null_size=1,ptr_width=32)
    case("codex-layout-"+str(base),[oracle("a",8,"db"),oracle("a.b",8,"other")],layout,
         ({"a":"4671ca551ee57390d7f69d362e83180a1f3b29a3b8d2f45e2cd80962c6692aec","a.b":pin},op))
# Dotted region names can render colliding paths across owners. Filter first.
for a, b in [("a","a.c"),("a.b","a.b.c"),("a.b.c","a.b.c.c")]:
    for base in (32,48):
        layout = dict(A_tab=1,H=8,S_cells=0,globals={},heap_base=128,heap_limit_max=0,
                      interface_regions={a:{"c.db":{"base":16}},b:{"db":{"base":base}}},
                      null_size=1,ptr_width=32)
        case(f"deep-{a}-{base}",[oracle(a,8,"c.db"),oracle(b,8,"db")],layout)
(ROOT/"support/r11_cases.json").write_text(j(cases)+"\n")
