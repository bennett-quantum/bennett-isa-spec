"""Independent R16 witness/default and §9.2 pins; no interpreter/rehash."""
import json
from pathlib import Path
from r15_pins import IH, OH, envelope, j, sha

root = Path(__file__).resolve().parent
case = next(c for c in json.loads((root / 'r13_schema_cases.json').read_text())
            if c['name'] == 'schedule-cleanupFalse')
prefix, line = case['object'].split('section schedule\n')
schedule = json.loads(line)
del schedule['bindings']['expansion']
text = prefix + 'section schedule\n' + j(schedule) + '\n'
assert text == (root / 'r16-default-expansion.bisa').read_text()
assert sha(text) == '8344b91cdba546f8bf29db14c4c239011cd088660863e015727c76ea8df25bfa'
assert schedule['bindings']['budget'] == [3, 0, 0, 0]
assert sha(j(schedule)) == 'd5ba44577e812d158265a048d8e4f8843b5a472cad120f47a893c6bbc9dfb99d'
# §9.2 excludes schedule including its boundary LF. Inc8 has no references.
assert sha(text.split('\nsection schedule\n')[0]) == OH
assert len(envelope.encode()) == 261 and sha(envelope) == IH
assert json.loads(text.split('section symbols\n')[1].splitlines()[0])['0']['interface_hash'] == IH
print(f'R16 file {sha(text)}, interface/closure {IH}, object {OH}, schedule {sha(j(schedule))}')
