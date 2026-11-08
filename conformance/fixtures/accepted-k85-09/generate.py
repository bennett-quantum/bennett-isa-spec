#!/usr/bin/env python3
"""Publish independently derived §12.2 fixed-image relations and rejections.

The reference CLI supplies executable identities only. No expected verdict,
tuple, coverage or matrix is read from the implementation under test.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def generate(binary, temporary):
    baseline = (HERE.parent / "accepted-k84-09/baseline.bisa").read_text()
    template = json.loads(baseline.split("section certificate\n")[1])
    files, cases = {}, []

    def publish(name, base, cert, outcome, derivation, elements=None, denominator=None,
                fraction=None, clause="§12.2 K85"):
        text = base.rstrip() + "\n\nsection certificate\n" + canonical(cert) + "\n"
        filename = name + ".bisa"
        files[filename] = text
        cases.append(dict(name=name, object=filename, sha256=sha(text.encode()),
                          outcome=outcome, derivation=derivation, normative_clause=clause,
                          **({} if elements is None else dict(elements=elements,
                             full_domain_elements=str(denominator), covered_fraction=fraction))))
        # Admission and semantic replay are distinct surfaces: check validates
        # shape/hash/schema; the Rust integration test verifies every verdict.
        if outcome in ("reject_admission", "proof", "proof_relative_to_image", "evidence"):
            accepted = outcome != "reject_admission"
            case = dict(id="D-k85-" + name, suite="D", spec="ISA0.9 " + clause,
                        derivation=derivation + " CLI check covers admission; portable checker APIs cover replay.",
                        object="../../../fixtures/accepted-k85-09/" + filename,
                        command=["check", "{object}"],
                        expect=dict(exit=0, stdout_json=dict(status="ok")) if accepted else
                        dict(exit=1, stdout_empty=True, stderr_contains=[clause.split()[0]]))
            files["../../cases/D/" + case["id"] + "/case.json"] = json.dumps(
                case, ensure_ascii=False, indent=2) + "\n"

    def object_(width=8, role="in", dirty=False, second=False, phase=False):
        mem = [dict(A=1, W=width, role=role)]
        if second:
            mem.append(dict(A=0, W=1, role="in"))
        signature = dict(abi="0.9", args=[], bounds=[], **{"in": [1]},
                         kind="perm_action" if not phase else "action", mem=mem, mut=[1, 4], tier="q")
        symbol = dict(name="lookup", kind=signature["kind"], signature=signature,
                      interface_hash="0" * 64)
        suffix = ", aux:region[0,1] in" if second else ""
        body = "v = qload db a\ncx v[0] t\nqunload db a v"
        if dirty:
            body += "\nb = qborrow any 1\ncx b t\ncx b t\nqreturn b"
        if phase:
            body += "\ns t\ns t"

        def source():
            envelope = " ".join(f"{key}=0.9" for key in
                                ("isa", "object", "launch", "sched", "cert", "coverage", "target"))
            return ("bennett-object " + envelope + " ptr_width=32\n\nsection symbols\n"
                    + canonical({"0": symbol}) + "\n\nsection functions\n"
                    + f"{signature['kind']} @lookup (t:wire mut, f:wire[4] mut) (a:wire in) "
                    + f"(db:region[1,{width}] {role}{suffix}) () ()\n^entry():\n{body}\n"
                    + "\nsection layout\n" + canonical(dict(ptr_width=32, H=2, A_tab=1,
                       S_cells=0, null_size=1, heap_base=1, heap_limit_max=0,
                       globals={}, interface_regions={})) + "\n")

        def hashes(text):
            temporary.write_text(text)
            result = subprocess.run([binary, "hash", str(temporary)], capture_output=True, text=True)
            assert result.returncode == 0, (result.stderr, text)
            return json.loads(result.stdout)

        symbol["interface_hash"] = hashes(source())["symbols"]["lookup"]["interface_hash"]
        base = source()
        identity = hashes(base)
        closure = identity["symbols"]["lookup"]["closure_hash"]
        cert = copy.deepcopy(template)
        bindings = cert["bindings"]
        bindings.update(object_hash=identity["object_hash"], interface_hash=symbol["interface_hash"],
                        closure_hash=closure, reference_program_hash=closure,
                        dialect_mapping_version="fixed-images.1")
        record = next(iter(cert["regions"].values()))
        record.update(signature=signature, kind=signature["kind"])
        record["frame"]["slots"] = [
            dict(id="mut:0", role="mut", width=1, **{"class": "fast"}),
            dict(id="mut:1", role="mut", width=4, **{"class": "fast"}),
            dict(id="in:0", role="in", width=1, **{"class": "fast"}),
            dict(id="mem:0", role="mem_" + role, width=2 * width, **{"class": "fast"})]
        if second:
            record["frame"]["slots"].append(
                dict(id="mem:1", role="mem_in", width=1, **{"class": "fast"}))
        cert["regions"] = {closure: record}
        cert["composition"]["multiplicities"] = {}
        cert["producer"]["materialisation"]["pipeline"] = "fixed-images"
        return base, cert, record, closure

    def relation(record, closure, image="0102", width=8, controls=0, dirty=False,
                 omitted=None, second=False):
        raw = bytes.fromhex(image)
        packed = int.from_bytes(raw, "little")
        cells = [(packed >> (i * width)) & ((1 << width) - 1) for i in range(2)]
        ranges = [("t", 1, 2), ("f", 4, 16), ("a", 1, 2)]
        bundles = [dict(name="control", width=1, lo=0, hi=1) for _ in range(controls)]
        bundles += [dict(name=n, width=w, lo=0, hi=size - 1)
                    for n, w, size in ranges if n != omitted]
        images = [dict(name="db", A=1, W=width, image=image, sha256=sha(raw))]
        if second:
            # Deliberately reverse the producer's list; report order is mem:0,mem:1.
            images.insert(0, dict(name="aux", A=0, W=1, image="01", sha256=sha(b"\x01")))
        claim = dict(kind="exhaustive", invocation=dict(args=[], budget=["0"] * 4),
                     subject=dict(closure_hash=closure, variant="forward", inherited_controls=controls),
                     domain=dict(bundles=bundles, borrowed=[dict(name="dirty", width=1, lo=0, hi=1)]
                                 if dirty else [], regions=[], fixed_regions=images))

        def tuple_(c, t, f, a, b):
            return dict(bundles=[str(x) for x in c + [t, f, a]],
                        borrowed=[str(b)] if dirty else [],
                        regions=[[str(v) for v in cells]] + ([["1"]] if second else []))

        rows = []
        # §12.2: declared order, last component fastest. Fixed cells never
        # enter this Cartesian product. qload/cx/qunload gives t XOR db[a]&1.
        for control in range(1 << controls):
            c = [(control >> i) & 1 for i in reversed(range(controls))]
            for t in range(1 if omitted == "t" else 2):
                for f in range(1 if omitted == "f" else 16):
                    for a in range(1 if omitted == "a" else 2):
                        for b in range(2 if dirty else 1):
                            out = t ^ ((cells[a] & 1) if all(c) else 0)
                            rows.append(dict(before=tuple_(c, t, f, a, b),
                                             after=tuple_(c, out, f, a, b)))
        claim["expected"] = dict(table=rows)
        record["claims"] = [claim]
        return claim

    # Full D, two values, NOT is the basis permutation t -> t XOR 1.
    base, text = baseline.split("section certificate\n")
    full = json.loads(text)
    publish("full-d-x", base, full, "proof", "X exchanges 0 and 1; n=|D|=2.", 2, 2, "1/1")
    sampled = copy.deepcopy(full)
    claim = next(iter(sampled["regions"].values()))["claims"][0]
    claim.update(kind="sampled", sampler=dict(algorithm="xoshiro256**", seed=[str(1 << 57), "0", "0", "0"], count=2))
    # s1 is initially 0, then 2^57. Outputs are 0, rotl(5*2^57,7)*9=45,
    # hence two draws cover both bits but the result is evidence.
    publish("sampled-full-d", base, sampled, "evidence",
            "Two xoshiro256** draws from seed (2^57,0,0,0) are 0 and 45; low bits cover both values; sampling remains evidence.", 2, 2, "1/1")

    base, cert, record, closure = object_()
    claim = relation(record, closure)
    for image in ("0102", "0201"):
        relation(record, closure, image)
        publish("image-" + image, base, cert, "proof_relative_to_image",
                "t,f,a have 2*16*2=64 tuples; db has 65536 unrestricted images. "
                "qload/cx/qunload gives t XOR (db[a]&1), preserves f,a,db; n/|D|=64/4194304=1/65536.",
                64, 4194304, "1/65536")
    relation(record, closure)
    reference = copy.deepcopy(cert)
    for name, mutation in [
        ("swapped-hash", lambda c: c["domain"]["fixed_regions"][0].update(sha256=sha(bytes.fromhex("0201")))),
        ("stale-hash", lambda c: c["domain"]["fixed_regions"][0].update(sha256="0" * 64)),
        ("hex-text-hash", lambda c: c["domain"]["fixed_regions"][0].update(sha256=sha(b"0102"))),
        ("wrong-a", lambda c: c["domain"]["fixed_regions"][0].update(A=0)),
        ("wrong-w", lambda c: c["domain"]["fixed_regions"][0].update(W=7)),
        ("uppercase", lambda c: c["domain"]["fixed_regions"][0].update(image="AB02")),
        ("short-image", lambda c: c["domain"]["fixed_regions"][0].update(image="01")),
        ("long-image", lambda c: c["domain"]["fixed_regions"][0].update(image="010200")),
        ("duplicate-image", lambda c: c["domain"]["fixed_regions"].append(copy.deepcopy(c["domain"]["fixed_regions"][0]))),
        ("duplicate-bundle", lambda c: c["domain"]["bundles"].append(copy.deepcopy(c["domain"]["bundles"][0]))),
        ("also-ranged", lambda c: c["domain"]["regions"].append(dict(name="db", cells="all"))),
    ]:
        mutated = copy.deepcopy(reference)
        mutation(next(iter(mutated["regions"].values()))["claims"][0])
        publish(name, base, mutated, "reject_admission", "Only the named fixed-image binding is invalid: " + name + ".")
    for omitted, count in (("t", 32), ("f", 4), ("a", 32)):
        relation(record, closure, omitted=omitted)
        divisor = 4194304 // count
        publish("omit-" + omitted, base, cert, "evidence",
                "Omitted " + omitted + " is zero by the existing slice rule; its full range has more than one value.",
                count, 4194304, "1/" + str(divisor))
    relation(record, closure)
    for label, outcome in (("proof", "reject_replay"), ("proof_relative_to_image", "proof_relative_to_image"),
                           ("evidence", "proof_relative_to_image")):
        claim = record["claims"][0]
        claim["label"] = label
        publish("request-" + label, base, cert, outcome,
                "Requested label " + label + " is checked against the exclusive image-relative outcome.",
                **({} if outcome == "reject_replay" else dict(elements=64, denominator=4194304, fraction="1/65536")))
    for name, settings, image, width, controls, dirty, second, count, bits in [
        ("controlled-dirty", dict(dirty=True), "0102", 8, 1, True, False, 256, 24),
        ("external-reference", dict(width=1, dirty=True), "01", 1, 1, True, False, 256, 10),
        ("packing", dict(width=5), "1f01", 5, 0, False, False, 64, 16),
        ("report-order", dict(second=True), "0102", 8, 0, False, True, 64, 23),
        ("mut-region", dict(role="mut"), "0102", 8, 0, False, False, None, None),
        ("nonzero-padding", dict(width=3), "d1", 3, 0, False, False, None, None),
        ("phase", dict(phase=True), "0102", 8, 0, False, False, None, None),
    ]:
        b, c, r, h = object_(**settings)
        relation(r, h, image, width, controls, dirty, second=second)
        outcome = "reject_admission" if name in ("mut-region", "nonzero-padding") else (
            "reject_replay" if name == "phase" else "proof_relative_to_image")
        publish(name, b, c, outcome,
                {"controlled-dirty": "Both controls and dirty bit range fully; two CX(b,t) cancel exactly, including entangled references. 256/2^24=1/65536.",
                 "external-reference": "The two one-bit cells are (1,0); output t XOR (control AND NOT a). Dirty/reference systems are restored. 256/1024=1/4.",
                 "packing": "0x011f = 31 + (8<<5); two 5-bit cells straddle a byte. Padding is zero. 64/2^16=1/1024.",
                 "report-order": "Producer lists aux before db; report uses mem:0 db, mem:1 aux. Fixed 17 bits give 1/131072.",
                 "mut-region": "An interface mut-role region cannot be fixed.",
                 "nonzero-padding": "Two 3-bit cells use six bits; 0xd1 has nonzero unused bits 6,7, even with a matching raw hash.",
                 "phase": "S^2 on t contributes -1 on the output-one rows; a permutation assertion requires exact +1."}[name],
                **({} if count is None else dict(elements=count, denominator=1 << bits,
                                                 fraction="1/" + str((1 << bits) // count))))
    b, c, r, h = object_(second=True)
    cl = relation(r, h, second=True)
    cl["domain"]["fixed_regions"] = [cl["domain"]["fixed_regions"][1]]
    for name, regions in [
        ("duplicate-cell", [dict(name="aux", cells=[dict(index=0, lo=0, hi=1)] * 2)]),
        ("duplicate-region", [dict(name="aux", cells="all")] * 2),
    ]:
        cl["domain"]["regions"] = regions
        publish(name, b, c, "reject_admission", "A remaining varying " + name.replace("duplicate-", "") + " cannot occur twice.")
    # A process-map vector, explicitly separate from executable certificates:
    # Z|0>=I|0>, but their unrestricted Frobenius difference is 2. Both
    # uncontrolled channels for -I and I agree, while controlled maps differ.
    files["restricted-map.json"] = canonical(dict(
        format="bennett.fixed-image-map.v1", isa="0.9", spec="§12.1 K85",
        U=[[1, 0], [0, -1]], V=[[1, 0], [0, 1]], J=[[1], [0]],
        input_dimension=1, output_dimension=2,
        restricted_frobenius_squared=0, unrestricted_frobenius_squared=4,
        restricted_choi_difference=[[0, 0], [0, 0]],
        strict_dual=dict(Y=[["0.125", "0"], ["0", "0.125"]], mu="0.375", bound="0.375"),
        derivation="(Z-I)J=0 at fixed |0>; Choi difference is 2x2, input dimension 1, full output dimension 2. "
                   "Y=I2/8 has pivots 1/8; Tr_out(Y)=1/4, mu=3/8 leaves pivot 1/8. "
                   "At |1>, Z-I=-2, so unrestricted Frobenius norm is 2. "
                   "This is independent map data, not an executable numerical certificate; that checker is staged.")) + "\n"
    files["k85.json"] = canonical(dict(format="bennett.fixed-images.v1", isa="0.9", cases=cases)) + "\n"
    return files


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--ref", default=os.environ.get("BENNETT_CONFORMANCE_BIN", str(
        ROOT / "reference-interpreter/target/debug/bennett-isa-ref")))
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix="fixed-images-", dir=os.environ["TMPDIR"]) as directory:
        files = generate(args.ref, Path(directory) / "object.bisa")
    for name, text in files.items():
        path = HERE / name
        if args.check:
            assert path.read_text() == text, f"stale vector: {path}"
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
    print(f"Fixed images: {len(files)} independently derived artifacts checked")


if __name__ == "__main__":
    main()
