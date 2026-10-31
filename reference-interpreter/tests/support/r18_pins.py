"""Independent literal R18 pins, no interpreter or rehash helper."""
import hashlib
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
fixture = (HERE.parent / "fixtures/nested.bisa").read_text()
pins = {
    2: "07c617a312f2b1f333689d95b474ccf9b7dea4a570c43ced87a92c089b66d9e7",
    3: "0584dc9a6dbe02b138e516caca59168a4dbbf577185e9d96504f58d5a5fcf4ae",
}
base = None
for budget, pin in pins.items():
    data = (HERE / f"r18-nested-budget-{budget}.bisa").read_bytes()
    assert hashlib.sha256(data).hexdigest() == pin
    body, raw = data.decode().split("\nsection schedule\n")
    assert body == fixture
    schedule = json.loads(raw)
    canonical = json.dumps(schedule, sort_keys=True, separators=(",", ":"))
    assert raw == canonical + "\n"
    assert schedule["bindings"]["budget"] == [budget, 0, 0, 0]
    params = b'[{"depth":0,"heap":0,"stack":0,"steps":3}]'
    assert hashlib.sha256(params).hexdigest() == schedule["bindings"]["params_hash"]
    assert len(schedule["entries"]) == 2
    assert schedule["entries"][0] == schedule["entries"][1]
    if base is None:
        base = schedule
    else:
        base["bindings"]["budget"][0] = budget
        assert base == schedule
    print(f"R18 B={budget}: file={pin}, schedule={hashlib.sha256(canonical.encode()).hexdigest()}")
print("Nested fixture bytes, canonical schedules and token preimage pins agree.")
