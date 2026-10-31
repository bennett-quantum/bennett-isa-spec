"""Independently check R15 witness bytes and §9.2 pins; no interpreter/rehash."""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
IH = '8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93'
OH = '8093da48bad04d6564325faf01ea1304a84b5ac5a120e685eee9951f5daec5d9'


def sha(text):
    return hashlib.sha256(text.encode()).hexdigest()


def j(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'))


# Literal, LF-terminated inc8 envelope. No references: closure = interface.
envelope = (
    'isa=0.8\nptr_width=32\n'
    '{"abi":"0.8","args":[],"fault":4,"in":[8],"kind":"oracle",'
    '"mem":[],"tgt":8,"tier":"q"}\n'
    'oracle @inc8 (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()\n'
    'regs { one:reg[8], r:reg[8] }\n^entry():\n'
    'one = lift.const.i8 1\nr = lift.add x one\nlift.retval r\n'
)
assert len(envelope.encode()) == 261
assert sha(envelope) == IH
case = next(c for c in json.loads((ROOT / 'r13_schema_cases.json').read_text())
            if c['name'] == 'schedule-cleanupFalse')
prefix, schedule = case['object'].split('section schedule\n')
schedule = json.loads(schedule)
schedule['bindings']['epsilon'] = '0.0'
epsilon = prefix + 'section schedule\n' + j(schedule) + '\n'
assert epsilon == (ROOT / 'r15-witnesses/r15-epsilon-0.0.bisa').read_text()
assert sha(epsilon) == '1431a1dbbe4991d170e3848995026610ebbffce08e7279ee05a68880f06c82df'
for path in sorted((ROOT / 'r15-witnesses').glob('*.bisa')):
    text = path.read_text()
    section = 'certificate' if 'frame' in path.name else 'schedule'
    # §9.2 excludes these sections, including their leading boundary LF.
    base, _ = text.split('\nsection ' + section + '\n')
    assert sha(base) == OH
    symbol = json.loads(text.split('section symbols\n')[1].splitlines()[0])['0']
    assert symbol['interface_hash'] == IH
    print(f'{path.name}: file {sha(text)}, interface/closure {IH}, object {OH}')
