#!/usr/bin/env python3
"""Generate k82.json from spec/ref-0.9.json with no interpreter imports. Run with python3 -I; --check compares bytes.

Independent of every expander: it sweeps the catalog for its named interfaces and
invoke edges, and expands the adapter edges itself to digests a consumer must match.
"""
import hashlib, json, pathlib, re, sys

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[2]
PIN = "5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197"
raw = (ROOT / "spec/ref-0.9.json").read_bytes()
envelope = b"isa=0.9\nexpansion=ref\nversion=0.9\n" + raw + b"\n"
digest = hashlib.sha256(envelope).hexdigest()
assert digest == PIN, digest
RECIPES = {r["kind"]: r for r in json.loads(raw)["recipes"]}
assert len(RECIPES) == 219
OPEN = sorted(k for k, r in RECIPES.items() if all(isinstance(p, str) for p in r["parameters"]))
SUB = re.compile(r"^([^\[]+)\[(.*)\]$", re.S)
OPS = {"+": lambda a, b: a + b, "-": lambda a, b: a - b, "*": lambda a, b: a * b,
       "div": lambda a, b: a // b, "mod": lambda a, b: a % b, "bit": lambda a, b: (a >> b) & 1}


def scalar(x, ints):
    if isinstance(x, int):
        return x
    if isinstance(x, str):
        return ints[x]
    return OPS[x[0]](*[scalar(y, ints) for y in x[1:]])


def split(w):
    m = SUB.match(w)
    return (m.group(1), json.loads(m.group(2))) if m else (w, None)


def interface(kind):
    """Names used before any binding in body order; open recipes hold only these constructors."""
    used = []

    def go(body, bound):
        for c in body:
            if "gate" in c:
                for w in c["wires"]:
                    n, _ = split(w)
                    if n not in bound and n not in used:
                        used.append(n)
            elif "alloc" in c:
                bound.add(c["alloc"])
            elif "release" in c:
                assert c["release"] in bound, (kind, c)
            elif "for" in c:
                go(c["body"], bound | {c["for"]})
            elif "inverse" in c:
                go(c["body"], bound)
            else:
                raise AssertionError(f"open recipe {kind}: unexpected constructor {sorted(c)}")

    ints = {p for p in RECIPES[kind]["parameters"]}
    go(RECIPES[kind]["body"], set(ints))
    return sorted(used)


def invokes(j):
    if isinstance(j, dict):
        if "invoke" in j:
            yield j
        for v in j.values():
            yield from invokes(v)
    elif isinstance(j, list):
        for v in j:
            yield from invokes(v)


def expand(kind, args):
    """Expand a catalog typed adapter over for/gate/alloc/release/inverse/invoke. Alloc identities are #k in order."""
    out = []
    serial = [0]

    def emit(gate, wires):
        out.append([gate, wires])

    def wire(w, env):
        n, sub = split(w)
        lst = env["wires"][n]
        if sub is None:
            return list(lst)
        i = scalar(sub, env["ints"])
        assert 0 <= i < len(lst), (kind, w, i, len(lst))
        return [lst[i]]

    def declare(body, table):
        for c in body:
            if "alloc" in c:
                table[c["alloc"]] = c
            if "body" in c:
                declare(c["body"], table)

    def acquire(name, width, env):
        ident = serial[0]
        serial[0] += 1
        env["wires"][name] = [f"#{ident}[{i}]" for i in range(width)]
        env["alloc"][name] = f"#{ident}"
        emit("alloc", [f"#{ident}"])

    def walk(body, env, rev):
        """K19 inverse: traverse constructors in reverse, exchanging allocation and release."""
        declare(body, env["decl"])
        for c in reversed(body) if rev else body:
            if "gate" in c:
                assert c["gate"] in ("x", "cx", "ccx", "swap"), c
                emit(c["gate"], [x for w in c["wires"] for x in wire(w, env)])
            elif "alloc" in c or "release" in c:
                name = c.get("alloc", c.get("release"))
                if ("alloc" in c) != rev:
                    acquire(name, scalar(env["decl"][name]["width"], env["ints"]), env)
                else:
                    env["wires"].pop(name)
                    emit("release", [env["alloc"].pop(name)])
            elif "for" in c:
                span = range(scalar(c["from"], env["ints"]), scalar(c["to"], env["ints"]))
                for i in reversed(span) if rev else span:
                    env["ints"][c["for"]] = i
                    walk(c["body"], env, rev)
                env["ints"].pop(c["for"], None)
            elif "inverse" in c:
                walk(c["body"], env, not rev)
            elif "invoke" in c:
                callee = RECIPES[c["invoke"]]
                ps = callee["parameters"]
                assert all(isinstance(p, str) for p in ps), "this expander covers adapter-to-open edges only"
                ints = {p: scalar(a, env["ints"]) for p, a in zip(ps, c["args"], strict=True)}
                # K82: the interface resolves in the invoking scope; nothing else is imported.
                wires = {n: env["wires"][n] for n in interface(c["invoke"])}
                walk(callee["body"], {"ints": ints, "wires": wires, "alloc": {}, "decl": {}}, rev)
            else:
                raise AssertionError(sorted(c))

    r = RECIPES[kind]
    ints, wires = {}, {}
    for p, a in zip(r["parameters"], args, strict=True):
        if p["type"] == "integer":
            ints[p["name"]] = a
        else:
            assert p["type"] == "wire", p
            wires[p["name"]] = [f"{a}[{i}]" for i in range(scalar(p["width"], ints))]
    walk(r["body"], {"ints": ints, "wires": wires, "alloc": {}, "decl": {}}, False)
    return out


def formal_width(p, ints):
    return scalar(p["width"], ints)


def tuples(kind):
    r = RECIPES[kind]
    names = [p["name"] for p in r["parameters"] if p["type"] == "integer"]
    if names == ["N"]:
        return [{"N": n} for n in (1, 2, 3, 4)]
    assert names == ["N", "M"], names
    op = kind.split(".")[-1]
    # §3.3: zext/sext strictly widen, trunc strictly narrows.
    return [{"N": n, "M": m} for n in range(1, 5) for m in range(1, 5) if (m < n if op == "trunc" else n < m)]


edges = []
for caller, r in sorted(RECIPES.items()):
    for site in invokes(r["body"]):
        callee = site["invoke"]
        if callee not in OPEN:
            continue
        assert caller not in OPEN, (caller, callee)
        formals = {p["name"]: p for p in r["parameters"]}
        names = interface(callee)
        for n in names:
            assert formals[n]["type"] == "wire", (caller, n)
        cases = []
        for t in tuples(caller):
            args = [t[p["name"]] if p["type"] == "integer" else f"r_{p['name']}" for p in r["parameters"]]
            leaves = expand(caller, args)
            text = json.dumps(leaves, separators=(",", ":"), ensure_ascii=False).encode()
            cases.append({"args": args,
                          "bundles": {f"r_{p['name']}": formal_width(p, t) for p in r["parameters"] if p["type"] == "wire"},
                          "leaves": len(leaves),
                          "sha256": hashlib.sha256(text).hexdigest()})
        edges.append({"caller": caller, "callee": callee, "interface": names, "instances": cases})
assert len(edges) == 20, len(edges)
invoked = {e["callee"] for e in edges}


def rec(kind, parameters, body):
    return {"kind": kind, "parameters": parameters, "body": body, "scratch": [], "removable": [], "source": "K82 vector"}


def x(w):
    return {"gate": "x", "wires": [w]}


def each_bit(name, n="N"):
    return {"for": "i", "from": 0, "to": n, "body": [x(f'{name}["i"]')]}


I = {"name": "N", "type": "integer"}


def W(name, width):
    return {"name": name, "type": "wire", "width": width}


OPEN_Q = rec("open.q", ["N"], [each_bit("q")])
ANY = {"class": "any", "state": "clean"}
cases = [
    {"id": "named-interface-resolves-at-the-site", "recipes": [OPEN_Q, rec("use", [I, W("q", "N")], [{"invoke": "open.q", "args": ["N"]}])],
     "root": "use", "args": [2, "r"], "bundles": {"r": 2}, "expect": [["x", ["r[0]"]], ["x", ["r[1]"]]]},
    {"id": "typed-recipe-is-closed", "recipes": [rec("closed", [I], [each_bit("q")]), rec("use", [I, W("q", "N")], [{"invoke": "closed", "args": ["N"]}])],
     "root": "use", "args": [2, "r"], "bundles": {"r": 2}, "expect": "reject"},
    {"id": "unbound-at-the-site-rejects", "recipes": [OPEN_Q, rec("use", [I, W("p", "N")], [{"invoke": "open.q", "args": ["N"]}])],
     "root": "use", "args": [2, "r"], "bundles": {"r": 2}, "expect": "reject"},
    {"id": "integer-binding-is-not-a-wire", "recipes": [OPEN_Q, rec("use", [I, {"name": "q", "type": "integer"}], [{"invoke": "open.q", "args": ["N"]}])],
     "root": "use", "args": [2, 1], "bundles": {}, "expect": "reject"},
    {"id": "read-past-the-binding-rejects", "recipes": [OPEN_Q, rec("use", [I, W("q", 1)], [{"invoke": "open.q", "args": ["N"]}])],
     "root": "use", "args": [2, "r"], "bundles": {"r": 1}, "expect": "reject"},
    {"id": "wider-binding-reads-only-what-the-body-reads", "recipes": [OPEN_Q, rec("use", [I, W("q", 3)], [{"invoke": "open.q", "args": ["N"]}])],
     "root": "use", "args": [2, "r"], "bundles": {"r": 3}, "expect": [["x", ["r[0]"]], ["x", ["r[1]"]]]},
    {"id": "free-integer-is-never-imported", "recipes": [rec("open.k", ["N"], [each_bit("q", "K")]), rec("use", [I, {"name": "K", "type": "integer"}, W("q", "N")], [{"invoke": "open.k", "args": ["N"]}])],
     "root": "use", "args": [1, 1, "r"], "bundles": {"r": 1}, "expect": "reject"},
    {"id": "bare-template-formals-import-nothing", "recipes": [rec("use", [{"name": "T", "type": "templates", "length": 1}, W("b", 1)], [{"apply": "T", "index": 0, "args": [0]}])],
     "root": "use", "args": [[{"parameters": ["k"], "body": [x("b")]}], "a"], "bundles": {"a": 1, "b": 1}, "expect": [["x", ["b[0]"]]]},
    {"id": "captured-body-resolves-at-its-capture", "recipes": [
        rec("open.one", [], [x("q[0]")]),
        rec("inner", [{"name": "B", "type": "body"}, W("q", 1)], [{"splice": "B"}]),
        rec("use", [W("q", 1), W("o", 1)], [{"invoke": "inner", "args": [[{"invoke": "open.one", "args": []}], "o"]}])],
     "root": "use", "args": ["r", "s"], "bundles": {"r": 1, "s": 1}, "expect": [["x", ["r[0]"]]]},
    {"id": "interface-is-transitive", "recipes": [
        rec("open.one", [], [x("q[0]")]),
        rec("open.outer", [], [{"invoke": "open.one", "args": []}]),
        rec("use", [W("q", 1)], [{"invoke": "open.outer", "args": []}])],
     "root": "use", "args": ["r"], "bundles": {"r": 1}, "expect": [["x", ["r[0]"]]]},
    {"id": "enclosing-open-local-binds-nested-interface", "recipes": [
        rec("open.one", [], [x("q[0]")]),
        rec("open.outer", [], [{"alloc": "q", "width": 1, **ANY}, {"invoke": "open.one", "args": []}, x("q[0]"), {"release": "q"}])],
     "root": "open.outer", "args": [], "bundles": {}, "expect": [["alloc", ["#0"]], ["x", ["#0[0]"]], ["x", ["#0[0]"]], ["release", ["#0"]]]},
    {"id": "callee-local-is-not-captured-by-an-unrelated-caller-name", "recipes": [
        rec("open.local", [], [{"alloc": "z", "width": 1, **ANY}, x("z[0]"), x("z[0]"), {"release": "z"}]),
        rec("use", [W("z", 1)], [{"invoke": "open.local", "args": []}])],
     "root": "use", "args": ["r"], "bundles": {"r": 1}, "expect": [["alloc", ["#0"]], ["x", ["#0[0]"]], ["x", ["#0[0]"]], ["release", ["#0"]]]},
    {"id": "live-declaration-binds-by-its-wire-actual-rule", "recipes": [
        rec("open.one", [], [x("q[0]")]),
        rec("use", [], [{"declare": "q", "width": 1, "class": {"class": "any"}, "state": "clean", "body": [{"acquire": "q"}, {"invoke": "open.one", "args": []}, x("q[0]"), {"relinquish": "q"}]}])],
     "root": "use", "args": [], "bundles": {}, "expect": [["alloc", ["#0"]], ["x", ["#0[0]"]], ["x", ["#0[0]"]], ["release", ["#0"]]]},
    {"id": "inactive-declaration-rejects", "recipes": [
        rec("open.one", [], [x("q[0]")]),
        rec("use", [], [{"declare": "q", "width": 1, "class": {"class": "any"}, "state": "clean", "body": [{"invoke": "open.one", "args": []}]}])],
     "root": "use", "args": [], "bundles": {}, "expect": "reject"},
    {"id": "region-binds-by-its-wire-actual-rule", "recipes": [
        rec("open.two", [], [x("q[0]"), x("q[1]")]),
        rec("use", [{"name": "q", "type": "region", "address": 1, "cell": 1, "role": "mut"}], [{"invoke": "open.two", "args": []}])],
     "root": "use", "args": [{"address": 1, "cell": 1, "role": "mut", "origin": "interface:m", "cells": "r"}], "bundles": {"r": 2},
     "expect": [["x", ["r[0]"]], ["x", ["r[1]"]]]},
    {"id": "each-binding-is-a-wire-binding", "recipes": [
        rec("open.one", [], [x("q[0]")]),
        rec("use", [W("w", 2)], [{"each": "q", "in": "w", "body": [{"invoke": "open.one", "args": []}]}])],
     "root": "use", "args": ["r"], "bundles": {"r": 2}, "expect": [["x", ["r[0]"]], ["x", ["r[1]"]]]},
]
for case in cases:
    # §11.1: recipes are in bytewise kind order.
    case["recipes"].sort(key=lambda r: r["kind"].encode())
doc = {
    "schema": "bennett.k82.v1",
    "catalog": {"envelope_sha256": digest, "recipes": len(RECIPES), "named_interface_recipes": len(OPEN)},
    "interfaces": {k: interface(k) for k in OPEN},
    "uninvoked_named_interface_recipes": sorted(set(OPEN) - invoked),
    "edges": edges,
    "cases": cases,
}
text = json.dumps(doc, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n"
target = HERE / "k82.json"
if "--check" in sys.argv:
    assert target.read_text(encoding="utf-8") == text, "k82.json differs from the generator"
    print(f"OK: {len(OPEN)} named-interface recipes, {len(edges)} edges, {sum(len(e['instances']) for e in edges)} instances, {len(cases)} cases")
else:
    target.write_text(text, encoding="utf-8")
    print(f"wrote {target}: {len(text)} bytes")
