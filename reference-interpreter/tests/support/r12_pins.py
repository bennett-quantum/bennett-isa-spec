"""§9.2 hand-assembled R12 envelopes; hashlib/json only, no interpreter.

CC/R: scalar leaves then BOTH region leaves, sorted by dotted path and original
components. Oracle: scalars plus exact owner. Definitions stay literal; no
references in B01. All strings below include the final LF.
"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
def j(v):
    return json.dumps(v, sort_keys=True, separators=(",", ":"))
def sha(s):
    return hashlib.sha256(s.encode()).hexdigest()
HEADER = "bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n"
cases = {}
def record(tag, funcs, layout, expected=None, object_pin=None):
    hashes, envelopes, symbols = {}, {}, {}
    for n, sig, body in funcs:
        env = "isa=0.8\nptr_width=32\n"+j(sig)+"\n"+body
        leaves = [((k,),v) for k,v in layout.items() if k not in ("globals","interface_regions")]
        leaves += [(("globals",k),v) for k,v in layout["globals"].items()]
        leaves += [(("interface_regions",owner,r),v)
                   for owner,rs in layout["interface_regions"].items()
                   for r,v in rs.items() if sig["kind"] != "oracle" or owner == n]
        env += "".join("layout "+".".join(k)+" "+j(v)+"\n"
                       for k,v in sorted(leaves,key=lambda kv:(".".join(kv[0]),kv[0])))
        hashes[n], envelopes[n] = sha(env), env
        symbols[str(len(symbols))] = dict(name=n,kind=sig["kind"],signature=sig,interface_hash=sha(env))
    obj = HEADER+"\nsection symbols\n"+j(symbols)+"\n\nsection functions\n"+"".join(f[2] for f in funcs)+"\nsection layout\n"+j(layout)+"\n"
    for n,p in (expected or {}).items():
        assert hashes[n] == p, (tag,n,hashes[n],p)
    if object_pin:
        assert sha(obj) == object_pin, (tag,sha(obj),object_pin)
    cases[tag] = dict(text=obj,hashes=hashes,envelopes=envelopes,object_hash=sha(obj))
    print(tag, hashes, sha(obj))

def oracle(n,r):
    sig = dict(abi="0.8",args=[],fault=4,**{"in":[8]},kind="oracle",mem=[dict(A=0,W=8,role="in")],tgt=8,tier="q")
    return n,sig,f"oracle @{n} (x:wire[8] in) (t:wire[8]) (f:wire[4]) ({r}:region[0,8] in) ()\n^entry():\nlift.retval x\n"
def layout(regions,sol=False):
    return dict(A_tab=1 if sol else 0,H=8 if sol else 6,S_cells=0,globals={},heap_base=128 if sol else 48,heap_limit_max=0,interface_regions=regions,null_size=1,ptr_width=32)
def main(tier):
    return "main",dict(abi="0.8",kind=tier,params=[],results=[],tier=tier),f"fn {tier} @main () () () () ()\n^entry():\nret\n"
for tier,h,op in [
    ("cc","a298a1060aa5dcf96e3e0d1c0d5251e6ad1acd44c776cf88caa9d25b6981c7bd","d478a37dbfa4895d6ba5f9673be187a4f5b9d75d6f38a11439208af9cddd0bc6"),
    ("r","8247f2deedc359e973521302ed25b78a21eadb8c169d01b60b3ee8d4030c48a3","d955f72740664092cdbfcf9c5c6541c9606895cfd17b9329ebc8db82e20091e4")]:
    record("codex-"+tier,[main(tier),oracle("a","b.c"),oracle("a.b","c")],layout({"a":{"b.c":{"base":16}},"a.b":{"c":{"base":16}}}),
           {"main":h,"a":"1e89833be7c3706227399760aff6daba246e541097163dc09d782d3d64fa0840","a.b":"e41c63761675397b0e898446e815def95046dd4e42017dd24b7e855b009592f4"},op)
for base,ah,cc,r,op in [
    (16,"d7b1663ea88edb0facd7401f8dc732a03facf5f9005901844b37f567a9f60afe","84b403e0032cfdc98e4c33e628c9662043833876045378f733aaaba7d311679e","68943b7ebec4b123a9b0aed04af75e1e037545794a3dee8a3aaceb7b623d421a","c7a1750c6e3415cb18ae1d699ba36067f0a382a93b6e409252df9bf0f1ee7f5d"),
    (17,"47b5d9e3495a9bae55e6e1b6211f6f52fc9fc7ca1e3cbe21de8592b9083e3b74","69d94d8f25dae1444c6ac6b3f46fe0e9bec110393405e778a84415913ee47bff","54af133bf40f7db2ca5da434db35f34f2157ae1808b46ed80137481e4a9efc17","483ad27610edeecc63993cb5891df642ec36f035920250f3ddcadefe7e6d814e")]:
    for tier,h in [("cc",cc),("r",r)]:
        record(f"sol-{tier}-{base}",[oracle("a","c.db"),oracle("a.c","db"),main(tier)],layout({"a":{"c.db":{"base":base}},"a.c":{"db":{"base":32}}},True),
               {"main":h,"a":ah,"a.c":"3c360ffb446a6418133d64bd4dddef00368d1f4c8f3b4723cb46f79262fd6f2a"},op if tier=="cc" else None)
# Depth 2–3 dotted names; equal / unequal / descending values prove the tie
# order follows original components, never value order. Each leaf changes alone.
for owner in ["a", "a.b", "a.b.c"]:
    for tier in ["cc","r"]:
        for b1,b2 in [(16,16),(16,32),(32,16),(17,32),(16,33)]:
            record(f"family-{owner}-{tier}-{b1}-{b2}",[oracle(owner,"c.db"),oracle(owner+".c","db"),main(tier)],layout({owner:{"c.db":{"base":b1}},owner+".c":{"db":{"base":b2}}},True))
(ROOT/"support/r12_cases.json").write_text(j(cases)+"\n")
