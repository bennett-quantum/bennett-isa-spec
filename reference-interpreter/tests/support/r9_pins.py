"""Independent ISA 9.2 derivation; writes only the four literal R9 fixtures."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "fixtures"
def j(x):
    return json.dumps(x, sort_keys=True, separators=(",", ":"))
def sha(s):
    return hashlib.sha256(s.encode()).hexdigest()
OS = {"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}
AS = {"abi":"0.8","args":["token"],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}
SOL = "oracle @o (p:wire in) (v:wire) (fr:wire[4]) () ()\n^entry():\nlift.retval p\n"
CODEX = "oracle @o (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x\n"
HEADER = "bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n"
BORROW = "b = qborrow any 1\ncx c b\nqreturn b\n"
CASES = [
    ("sol-oracle", SOL, "c = qalloc any 1\nf = qalloc any 4\nh c\noracle_call @o tk (c) t f () ()\noracle_call @o tk (c) t f () ()\nh c\n" + BORROW + "qfree f\nqfree c\n", "4cabd490b0d1a8dfe31489ec234e91867e66b54c6c4d030eb4e8d1cf6f84bb38", "ef010633dbc984abbaa9fbfb97c6233c5390fa99a17bc38e09725cddec936832"),
    ("sol-qinplace", SOL, "c = qalloc any 1\nh c\nqinplace @o @o tk c\nqinplace @o @o tk c\nh c\n" + BORROW + "qfree c\n", "bc6e5d89276a62eec7074a716d8dbce40e94a2c80c287ad8fc5b5e5c6c756fbf", "77664ac9392aec50b0415027771ae7ef49dcea7762b70025f0cd918d53a6ee1e"),
    ("codex-oracle", CODEX, "c = qalloc any 1\nd = qalloc any 1\nf = qalloc any 4\nh c\noracle_call @o tk (c) d f () ()\noracle_call @o tk (c) d f () ()\nh c\n" + BORROW + "qfree f\nqfree d\nqfree c\n", "be1f0e087e06fecd99df7c3cbb5c3c8c186fde8f814be1f826487e858fa66296", "49ad3ba6c411ed73bebacdebe6728ebff5995dbf8bbc03c016d1cb0c5c3e8625"),
    ("loop", None, "c = qalloc any 1\nqloop 2 {\n" + BORROW + "qfree c\nc = qalloc any 1\nh t\nh t\n}\nqfree c\n", "bb983f1dc5a8eea46912ea64d1226a9345a05d6cba6cc8452efc9f39b4a17a89", "80478acc73d52bbf64c395816470293b5b4b4a1ee36a5c550bf00436f06fc192"),
]
for name, oracle, body, expected_h, expected_object in CASES:
    sig = AS if oracle else dict(AS, args=[], bounds=[2])
    function = "action @a (t:wire mut) () () " + ("(tk:token) ()\n" if oracle else "() ()\nregs {  } bounds {2}\n") + "^entry():\n" + body
    refs = []
    if oracle:
        oh = sha("isa=0.8\nptr_width=32\n" + j(OS) + "\n" + oracle)
        assert oh == ("a8f4a15663aead9ecb6618fe3847deba546cf3172f645ffc7e763f551e529e27" if oracle == SOL else "cab76c914fa09a3fc8237381c6de91e83eeaefbd65cb7f52571ed4020abe8926")
        refs = [oh]
    envelope = "isa=0.8\nptr_width=32\n" + j(sig) + "\n" + function.replace("@o", "@" + refs[0] if refs else "@o") + "".join(h + "\n" for h in refs)
    h = sha(envelope)
    assert h == expected_h, (name, h)
    symbols = {}
    if oracle:
        symbols["0"] = dict(name="o", kind="oracle", signature=OS, interface_hash=oh)
    symbols[str(len(symbols))] = dict(name="a", kind="action", signature=sig, interface_hash=h)
    text = HEADER + j(symbols) + "\n\nsection functions\n" + (oracle or "") + function
    assert sha(text) == expected_object, (name, sha(text))
    (ROOT / ("r9-" + name + ".bisa")).write_text(text)
    comment = ("Hand: for B >= 1, @o is the one-step identity permutation, both qinplace rows are I, H^2=I, and CX(0,b)=I_bR. B=0 rejects under §4.5 because the oracle faults; it is not an identity run.\n"
               if name == "sol-qinplace" else
               "Hand: H; O; O†; H = I (oracle B=0 fault XORs also cancel); CX(0,b)=I_bR.\n"
               if oracle else
               "Hand: each visit returns b under clean c, frees c and allocates a fresh zero; H²=I_t.\n")
    (ROOT / ("r9-" + name + ".comments")).write_text(comment)
    print(name, "H=" + h, "object=" + sha(text), "envelope-bytes=" + str(len(envelope.encode())))
