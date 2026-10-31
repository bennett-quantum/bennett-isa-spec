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

# Exact council two-function probes. These pin the binary64 and discrete cases
# independently of the larger control_flow fixture and every interpreter hash.
phase = "action @phase (t:wire mut) () () () ()\n^entry():\nrz 6.283185307179586 t\n"
discrete = "s t\ns t\nx t\ns t\ns t\nx t\n"
probe = "action @probe (t:wire mut, p:wire mut) () () () ()\n^entry():\nh p\nctl p action_call @phase (t) () () ()\nh p\n"
for tag, body, hp, ho, op in [
    ("rounded-probe",phase,"eb4b8c89a878e6fb8c514214417d0827f809fe0fd6b155e2f395ec19f5f31271","0655c9e45f09d1748fbf0241e6d38244e7f353153a3f365f0a5f712b37a6e24d","04baeadb96dbafb2dc94832982272b6cfeb60174de6c8bb407bd28a3e4c37294"),
    ("exact-probe",phase.replace("rz 6.283185307179586 t\n",discrete),"300340284f4927db0fc9ccf6b6285917606928b2926e62e015034eb0174e2713","0dcda336816f668d43eb75a083d0ee26574ccf854078a639de0a08a64f79009d","0b71a29321bd566442de1d6e61774603031c3616a0b1cb4875b7719016837457")]:
    case(tag,[("phase",action,body,[]),("probe",dict(action,mut=[1,1]),probe,["phase"])],
         expected=({"phase":hp,"probe":ho},op))

(ROOT/"support/r11_cases.json").write_text(j(cases)+"\n")

# Idempotently add minus_i while retaining the literal rounded phase. Only
# controlled_phase depends on minus_i. All other stored pins stay unchanged.
fixture = ROOT/"fixtures/control_flow.bisa"
text = fixture.read_text()
symbols = json.loads(text.splitlines()[3])
functions = text.split("section functions\n")[1].split("\nsection data\n")[0]
minus = "action @minus_i (t:wire mut) () () () ()\n^entry():\n"+discrete
if "action @minus_i " not in functions:
    functions = functions.replace("action @controlled_phase ",minus+"action @controlled_phase ")
functions = functions.replace("ctl p action_call @phase (t) () () ()","ctl p action_call @minus_i (t) () () ()")
minus_env = "isa=0.8\nptr_width=32\n"+j(action)+"\n"+minus
minus_hash = sha(minus_env)
control_sig = dict(action, **{"in":[1]})
control_body = "action @controlled_phase (t:wire mut) (p:wire in) () () ()\n^entry():\nctl p action_call @minus_i (t) () () ()\n"
control_env = "isa=0.8\nptr_width=32\n"+j(control_sig)+"\n"+control_body.replace("@minus_i ","@"+minus_hash+" ")+minus_hash+"\n"
for symbol in symbols.values():
    if symbol["name"]=="controlled_phase":
        symbol["interface_hash"] = sha(control_env)
if not any(s["name"]=="minus_i" for s in symbols.values()):
    symbols[str(len(symbols))] = dict(name="minus_i",kind="action",signature=action,interface_hash=minus_hash)
text = text.split("\nsection symbols\n")[0]+"\nsection symbols\n"+j(symbols)+"\n\nsection functions\n"+functions+"\nsection data\n"+text.split("\nsection data\n")[1]
fixture.write_text(text)
pins = dict(envelopes=dict(minus_i=minus_env,controlled_phase=control_env),
            hashes=dict(minus_i=minus_hash,controlled_phase=sha(control_env)),object_hash=sha(text))
(ROOT/"support/r11_control_flow_pins.json").write_text(j(pins)+"\n")
print("control_flow",pins["hashes"],pins["object_hash"])
