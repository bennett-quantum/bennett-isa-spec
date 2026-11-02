#!/usr/bin/env python3
"""Bounded document/citation validation; no product or conformance execution."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import resource

_cap = 256 * 1024 * 1024
_soft, _hard = resource.getrlimit(resource.RLIMIT_AS)
resource.setrlimit(resource.RLIMIT_AS, (min(_cap, _hard) if _hard != resource.RLIM_INFINITY else _cap, _hard))

ROOT = Path(__file__).resolve().parent
DOCS = ("AUDIT.md", "cross-repo-beads.md", "MIXED-BINARY-RUNG.md")
IDS = {f"F{i:02}" for i in range(1, 32)}
REF = re.compile(r"^\[([a-z0-9-]+)\]: (\S+)$", re.M)
LINK = re.compile(r"\]\(([^)\n]+)\)")
RANGE = re.compile(r"L(\d+)(?:-L(\d+))?")


def require(ok, message):
    if not ok:
        raise ValueError(message)


def check_documents():
    texts = {name: (ROOT / name).read_text() for name in DOCS}
    audit, beads, rung = (texts[name] for name in DOCS)
    rows = re.findall(r"^\| (F\d\d) · ([^|]+)\|(.+)$", audit, re.M)
    require(len(rows) == len(IDS) and {row[0] for row in rows} == IDS, "audit must contain exactly F01–F31")
    owners = {}
    for key, owner, tail in rows:
        cells = [v.strip() for v in tail.strip().strip("|").split("|")]
        require(len(cells) == 5, f"{key}: malformed table row")
        require(cells[2] in {"locks out", "makes harder", "neutral"}, f"{key}: invalid severity")
        require(all(cells), f"{key}: missing source/assumption/change/cost")
        owners[key] = owner.strip()
    bodies = re.split(r"^## (F\d\d) — [^\n]+\n", beads, flags=re.M)
    require(len(bodies) == 1 + 2 * len(IDS) and set(bodies[1::2]) == IDS, "need one bead body per audit row")
    for key, body in zip(bodies[1::2], bodies[2::2]):
        for field in ("Owner:", "Type:", "Priority:", "Labels:", "Parent context:", "Source:",
                      "Determining sentence:", "Requested change:", "Acceptance:", "Cheap now:",
                      "Dependencies:", "Filing:"):
            require(field in body, f"{key}: missing {field}")
        require(f"Owner: `{owners[key]}`" in body, f"{key}: owner mismatch")
        require("Labels: `fat-object`" in body, f"{key}: missing fat-object label")
        deps = re.search(r"^Dependencies: (.+)$", body, re.M)[1]
        require(set(re.findall(r"F\d\d", deps)) <= IDS, f"{key}: invalid dependency")
    require("Filing trigger:" in rung and "rung 3 or rung 4 lands" in rung, "missing deferred filing trigger")
    require("Status: proposal" in rung and "not a published contract" in rung, "rung status is ambiguous")
    require(len(re.findall(r"^\d+\. \*\*", rung, re.M)) == 9, "need nine required mixed-rung evidence rows")
    for term in ("native main", "argv/envp", "host-only libc", "both executors", "reversal",
                 "replay", "pointer-rich", "persistent", "certificate", "budget"):
        require(term in rung, f"rung missing {term}")
    refs = dict(REF.findall(audit))
    require(refs == dict(REF.findall(beads)), "audit/bead citation maps differ")
    sources = {}
    for name, text in texts.items():
        definitions = dict(REF.findall(text))
        # Only this audit's lowercase citation labels use reference links.
        for label in re.findall(r"\]\[([a-z][a-z0-9-]*)\]", text):
            require(label in definitions, f"{name}: undefined citation {label}")
        for target in [*definitions.values(), *LINK.findall(text)]:
            require("://" not in target, f"unexpected network link: {target}")
            rel, _, fragment = target.partition("#")
            path = (ROOT / rel).resolve()
            require(path.is_file(), f"{name}: missing link {target}")
            if path.parent == ROOT:
                continue
            data = path.read_bytes()
            item = sources.setdefault(rel, {"sha256": hashlib.sha256(data).hexdigest(), "ranges": {}})
            if fragment:
                match = RANGE.fullmatch(fragment)
                require(match is not None, f"unsupported source fragment {target}")
                start = int(match[1])
                end = int(match[2] or start)
                lines = data.decode().splitlines(keepends=True)
                require(1 <= start <= end <= len(lines), f"out-of-range citation {target}")
                item["ranges"][fragment] = "".join(lines[start - 1:end])
    return texts, sources


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--capture-sources", action="store_true", help="record reviewed citation excerpts")
    args = parser.parse_args()
    texts, sources = check_documents()
    snapshot = ROOT / "sources.json"
    if args.capture_sources:
        snapshot.write_text(json.dumps({"format": 1, "sources": sources}, indent=2, sort_keys=True) + "\n")
    saved = json.loads(snapshot.read_text())
    require(saved["format"] == 1, "unknown source-snapshot format")
    require(set(saved["sources"]) == set(sources), "source inventory changed; review and recapture")
    drift = []
    for name, actual in sources.items():
        prior = saved["sources"][name]
        require(prior["ranges"] == actual["ranges"], f"cited source changed: {name}; review lines before recapture")
        if prior["sha256"] != actual["sha256"]:
            drift.append(name)
    count = sum(len(v["ranges"]) for v in sources.values())
    print(f"PASS: {len(texts)} deliverables, {len(IDS)} audit rows/bead bodies, nine mixed-rung evidence rows, "
          f"{len(sources)} source files, {count} cited ranges; 256 MiB limit.")
    if drift:
        print("Source edits outside cited ranges (original SHA-256 retained): " + ", ".join(drift))
    print("Documentation/citation validation only; no implementation, interpreter or conformance gate executed.")


if __name__ == "__main__":
    main()
