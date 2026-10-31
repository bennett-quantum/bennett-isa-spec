"""Independent §9.2 envelopes for the ten displayed §16.3 objects (stdlib only)."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def jcs(value):
    # The examples contain only ASCII keys and integer JSON numbers.
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)


def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()


def examples(draft):
    section = draft.split("### 16.3 ", 1)[1].split("\n## 17.", 1)[0]
    objects = re.findall(r"```\n(bennett-object .*?)\n```", section, re.S)
    assert len(objects) == 10
    return objects


def hashes(source):
    sections = dict(re.findall(r"section (\w+)\n(.*?)(?=\n\nsection |\Z)", source, re.S))
    symbols = {s["name"]: s for s in json.loads(sections["symbols"]).values()}
    bodies = {}
    for body in re.split(r"\n(?=(?:fn |oracle |lproc |action |perm_action ))", sections["functions"]):
        name = re.search(r"@([\w.]+)", body)[1]
        bodies[name] = body
    data = dict(line.split(" ", 1) for line in sections.get("data", "").splitlines())
    layout = json.loads(sections.get("layout", "{}"))
    devices = json.loads(sections.get("devices", "{}"))
    leaves = []
    for key, value in layout.items():
        if key == "globals":
            leaves.extend(([key, name], leaf) for name, leaf in value.items())
        elif key == "interface_regions":
            leaves.extend(([key, name, region], leaf) for name, regions in value.items() for region, leaf in regions.items())
        else:
            leaves.append(([key], value))
    leaves.sort(key=lambda row: jcs(row[0]).encode())
    refs = {}
    for name in symbols:
        # Definition in the first line is not a reference.
        body = bodies.get(name, "")
        refs[name] = list(dict.fromkeys(n for n in re.findall(r"@([\w.]+)", body.partition("\n")[2]) if n in symbols))
    reach = {n: set(rs) for n, rs in refs.items()}
    for _ in symbols:
        for n in symbols:
            reach[n].update(*(reach[m] for m in list(reach[n])))
    result, envelopes, components = {}, {}, {}

    def component(name):
        members = sorted({name} | {n for n in reach[name] if name in reach[n]}, key=lambda n: n.encode())
        cyclic = len(members) > 1 or name in refs[name]
        for member in members:
            for dep in refs[member]:
                if dep not in members and dep not in result:
                    component(dep)
        local = {n: "$" + str(i) for i, n in enumerate(members)} if cyclic else {}
        for member in members:
            sym = symbols[member]
            kind = sym["kind"]
            body = bodies.get(member, member + " " + data.get(member, ""))
            first, sep, rest = body.partition("\n")
            body = first + sep + re.sub(r"@([\w.]+)", lambda m: "@" + (local[m[1]] if m[1] in local else result[m[1]]) if m[1] in symbols else m[0], rest)
            selected = []
            if kind == "oracle":
                selected = [(path, value) for path, value in leaves if path[0] != "interface_regions" or path[1] == member]
            elif kind in ("cc", "r"):
                selected = leaves
            elif kind == "lproc":
                globals_used = re.findall(r"lift\.addr @([\w.]+)", bodies[member])
                selected = [(path, value) for path, value in leaves if path[0] == "globals" and path[1] in globals_used]
            images = sorted({value["image"] for path, value in selected if isinstance(value, dict) and "image" in value}, key=lambda n: n.encode())
            lines = ["isa=" + re.search(r"isa=([0-9.]+)", source)[1], "ptr_width=" + re.search(r"ptr_width=(\d+)", source)[1], jcs(sym["signature"]), body]
            lines += ["data " + n + " " + data[n] for n in images]
            lines += ["layout " + jcs(path) + " " + jcs(value) for path, value in selected]
            if kind in ("cc", "r"):
                used = re.findall(r"effect ([\w.]+) ", bodies[member])
                used = {n.split(".")[0] for n in used}
                used.update(re.findall(r"hostcall @?([\w.]+)", bodies[member]))
                entries = {**devices.get("models", {}), **devices.get("services", {})}
                lines += ["device " + n + " " + jcs(entries[n]) for n in sorted(used)]
            lines += [result[n] for n in refs[member] if n not in local]
            envelopes[member] = "\n".join(lines) + "\n"
        digest = sha("".join(envelopes[n] for n in members))
        for i, member in enumerate(members):
            result[member] = sha(digest + ":" + str(i)) if cyclic else digest
            components[member] = (members, digest)

    for name in symbols:
        if name not in result:
            component(name)
    return result, envelopes, components


def check(draft):
    for index, source in enumerate(examples(draft), 1):
        computed, envelopes, _ = hashes(source)
        symbols = json.loads(source.split("section symbols\n")[1].split("\n")[0])
        for sym in symbols.values():
            assert sym["interface_hash"] == computed[sym["name"]], (index, sym["name"], computed[sym["name"]])
        if index in (1, 2):
            # Check each fully printed envelope too, including SCC substitution.
            name = "inc8" if index == 1 else "sum"
            assert "```\n" + envelopes[name] + "```" in draft
        print(f"example {index}: " + ", ".join(n + "=" + h for n, h in computed.items()))


if __name__ == "__main__":
    check((ROOT / "spec/Bennett-ISA-Spec-0.9-draft.md").read_text())
