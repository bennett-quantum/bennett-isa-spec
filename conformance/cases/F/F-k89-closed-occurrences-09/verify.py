"""Closed K89 byte productions and lifetime vectors; no reader verdict."""
from copy import deepcopy
import json
from pathlib import Path
import re

here = Path(__file__).resolve().parent
draft = (here.parents[3] / "spec/Bennett-ISA-Spec-0.9.md").read_text()
marker = "**K89-OCCURRENCE witness**\n\n```json\n"
assert marker in draft, "missing K89 closed occurrence publication"
vectors = json.loads((here / "occurrences.json").read_text())
assert vectors == json.loads(draft.split(marker, 1)[1].split("\n```", 1)[0])


def uint(value):
    return type(value) is int and value >= 0


def coordinate(value, edge=False):
    required = {"symbol", "point", "iteration", "instance"}
    if edge:
        required.add("call_index")
    assert type(value) is dict and required <= value.keys()
    assert value.keys() <= required | {"step", "phase"}
    assert re.fullmatch("[0-9a-f]{64}", value["symbol"])
    assert type(value["point"]) is str and value["point"] != "*"
    for name in ["iteration", "instance"]:
        assert type(value[name]) is list and all(map(uint, value[name]))
    if edge:
        assert uint(value["call_index"])
    assert ("step" in value) == ("phase" in value)
    if "step" in value:
        assert uint(value["step"]) and value["step"] > 0
        assert value["phase"] in ["fwd", "rev"]


def source(value):
    assert type(value) is dict and "context_key" in value
    terminal = dict(value)
    context = terminal.pop("context_key")
    assert type(context) is list
    for edge in context:
        coordinate(edge, edge=True)
    coordinate(terminal)


def allocation(value):
    assert type(value) is dict and value.keys() == {"source", "allocation_ordinal"}
    source(value["source"])
    assert uint(value["allocation_ordinal"])


frame = vectors["borrow_frame"]
source(frame["first"])
source(frame["last"])
source(vectors["nested"])
source(vectors["oracle"])
first = vectors["workspace"][0]["allocation"]
allocation(first)
allocation(vectors["reacquisition"])
assert frame["first"] == frame["last"] and frame["first"]["point"] == "entry:2"
assert first["source"]["point"] == "entry:0"
assert first["source"] == vectors["reacquisition"]["source"]
assert first != vectors["reacquisition"]
# At frame entry only the original lifetime is live; storage reuse cannot
# make a stale ordinal resolve to its successor (or vice versa).
live_at_entry = [first]
assert vectors["reacquisition"] not in live_at_entry
bad_sources = []
for key, value in [("context_key", "*"), ("instance", "*"), ("iteration", [-1]),
                   ("generated", 0), ("call_index", 0), ("phase", "fwd"),
                   ("step", 0), ("symbol", "F" * 64), ("point", "*")]:
    bad = deepcopy(frame["first"])
    bad[key] = value
    bad_sources.append(bad)
edge_missing_call = deepcopy(vectors["nested"])
del edge_missing_call["context_key"][0]["call_index"]
bad_sources.append(edge_missing_call)
for bad in bad_sources:
    try:
        source(bad)
    except (AssertionError, TypeError):
        continue
    raise AssertionError(f"malformed source occurrence accepted: {bad}")
for bad in [dict(first, allocation_ordinal=-1), dict(first, allocation_ordinal=True),
            {"source": first["source"]}, dict(first, storage_address=0)]:
    try:
        allocation(bad)
    except AssertionError:
        continue
    raise AssertionError(f"malformed allocation occurrence accepted: {bad}")
print("PASS: K89 closed source/allocation records, nested/step coordinates and reused lifetimes")
