"""Independent §9.2 literal envelopes, including rejected R10 grammar bytes.
Run this script to reproduce r10_cases.json; no interpreter calls.
"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
def j(x):
    return json.dumps(x, sort_keys=True, separators=(",", ":"))
def sha(s):
    return hashlib.sha256(s.encode()).hexdigest()
HEADER = "bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n"
cases = {}
def case(tag, funcs, expected=None):
    symbols, hashes = {}, {}
    for name, sig, body, refs in funcs:
        env = "isa=0.8\nptr_width=32\n" + j(sig) + "\n" + body
        for ref in refs:
            env = env.replace("@" + ref, "@" + hashes[ref])
        env += "".join(hashes[ref] + "\n" for ref in refs)
        h = sha(env)
        hashes[name] = h
        symbols[str(len(symbols))] = dict(name=name,kind=sig["kind"],signature=sig,interface_hash=h)
    text = HEADER + j(symbols) + "\n\nsection functions\n" + "".join(f[2] for f in funcs)
    result = dict(text=text, hashes=hashes, object_hash=sha(text))
    if expected:
        assert (hashes["a" if "a" in hashes else "inc8"], sha(text)) == expected, (tag, result)
    cases[tag] = result
    print(tag, hashes, sha(text))

cc = dict(abi="0.8",kind="cc",params=[],results=[],tier="cc")
def simple(tag, head, regs="", sig=cc, expected=None):
    body = head + " @a () () () () ()\n" + regs + "^entry():\nret\n"
    case(tag, [("a",sig,body,[])], expected)
simple("bare-cc","cc",expected=("2060222c2773451dd9a97e146d566bfca81c7a91c7fe04bd90d7e841163f531e","b6f4dc99c6f84d162c82c54ea4c41ceee3e9d4cc09a00c3ab8bd80d282faec3f"))
simple("cc","fn cc",expected=("218bf0ae43801ce9c291e2c3276d60df583a7a8b939b56312f88b1623c2e68cd","d2cb6bf361cd11bd9e68e31a0329b5b62bb9c2d23cf6cb7ca9d8ddf89bd208e5"))
simple("empty-bounds","fn cc","regs {  } bounds {}\n",expected=("cb5e25ba34a6c84c96766f19a4b30c0999b7f262c2df63ae436296ba4a2554c8","8f2d76a7e18603d59a53082cd0493913725397140a6cbd162b7150fdb5358f11"))
simple("omitted-bounds","fn cc","regs {  }\n",expected=("4b749e1b37d610bc72f3b4d5c63ea551364492a370bd2dc5054282c5b1b89e62","028699ab2f8bd8767b19d021e1153bac74c470d9a302636fe66928bcc0d74de5"))
simple("bare-r","r",sig=dict(cc,kind="r",tier="r"))
simple("r","fn r",sig=dict(cc,kind="r",tier="r"))
inc = (ROOT/"fixtures/inc8.bisa").read_text()
sig = json.loads(inc.splitlines()[3])["0"]["signature"]
body = inc.split("section functions\n")[1]
for tag, mutated, hp, op in [
    ("fn-oracle",body.replace("oracle @","fn oracle @"),"607bffa4376f76dc181141c19dc70dfbde0d8064e3d220187b13001e25cab0ca","3868eb11e0293db6f43d3a9c5589c36feab4d88916327a34241141c1949b2dd6"),
    ("inc-empty-bounds",body.replace("regs { one:reg[8], r:reg[8] }","regs { one:reg[8], r:reg[8] } bounds {}"),"31e7ad0837d13f0cd78e0fc2a155016c006ab18e3ecf6cc367308d58eb274437","6e0b6cfb1a74f8974706264ddc3b762bda7226e0abeb284f97978d5cb2f7d4f3"),
    ("empty-lhs",body.replace("lift.retval r","() = lift.retval r"),"a2f5278ca8db3abb0e407e475c1fba93c4307b97f0db8c1fe4a0bbe2b48da782","4ede077a7344efa101b22cd0f990feacde2c1a415dbec6667ee61290f9747480")]:
    case(tag,[("inc8",sig,mutated,[])],(hp,op))
case("tuple-lhs",[("inc8",sig,body.replace("r = lift.add","(r) = lift.add"),[])])
loop_sig=dict(abi="0.8",args=[],bounds=[1],**{"in":[]},kind="action",mem=[],mut=[1],tier="q")
case("one-trip",[("a",loop_sig,"action @a (t:wire mut) () () () ()\nregs {  } bounds {1}\n^entry():\nqloop 1 {\nx t\n}\n",[])])
old=(ROOT/"fixtures/r9-codex-oracle.bisa").read_text()
os=json.loads(old.splitlines()[3])["0"]["signature"]
oracle=old.split("section functions\n")[1].split("action @a")[0]
a_sig=dict(abi="0.8",args=["token"],bounds=[],**{"in":[1,1]},kind="action",mem=[],mut=[1],tier="q")
prefix="action @a (t:wire mut) (p:wire in, q:wire in) () (tk:token) ()\n^entry():\nc = qalloc any 1\nd = qalloc any 1\nf = qalloc any 4\nh c\n"
call="oracle_call @o tk (c) d f () ()\n"
suffix="h c\nb = qborrow any 1\ncx c b\nqreturn b\nqfree f\nqfree d\nqfree c\n"
def ctx(stack):
    return "".join("qpred "+s+"\n" for s in stack)+call+"".join("qunpred "+s+"\n" for s in reversed(stack))
for tag,first,second,expected in [
    ("controls",["p","q"],["q","p"],("386dce74595f0e05ed2b7eb9218fc22fece3fafe7ad6fd206aed542ad8b7deef","7b1153bae80cc041447870eaa9623f73e6cb6df5085365a1efcbac99789076ad")),
    ("duplicate",["p"],["p","p"],None)]:
    case(tag,[("o",os,oracle,[]),("a",a_sig,prefix+ctx(first)+ctx(second)+suffix,["o"])],expected)
(ROOT/"support/r10_cases.json").write_text(j(cases)+"\n")
