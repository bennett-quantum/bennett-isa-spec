"""Independent reconstruction of both R17 witnesses; no interpreter/rehash."""
import copy
import json
from pathlib import Path
from r15_pins import IH, OH, envelope, j, sha

root = Path(__file__).resolve().parent
corpus = json.loads((root / 'r13_schema_cases.json').read_text())
for case_name, filename, file_pin, schedule_pin in [
    ('schedule-gateFalse', 'r17-default-epsilon-share.bisa',
     '8af833c21de5d94c6684b36cf115c46f5080f528fc898ece2bdda9091b9fb2a5',
     'd9025ef24bfd8db01e4e3e0f5b28be776c4e5585b000fd5bd2333122c0d4845d'),
    ('schedule-cleanupFalse', 'r17-shadowed-tie.bisa',
     '9fc08dca3c5697d9fcad31bf82f4a4a566d0fda6959d10675859d748696c1aff',
     'e13ea989795d205fcf983b033f24bf1945d8be197c241b3a440a67b2489c287f'),
]:
    original = next(c['object'] for c in corpus if c['name'] == case_name)
    prefix, line = original.split('section schedule\n')
    schedule = json.loads(line)
    if 'epsilon-share' in filename:
        del schedule['entries'][0]['gate']['epsilon_share']
    else:
        a = schedule['entries'][0]
        a['path'][0]['iteration'] = '*'
        a['generated'] = 0
        b, c = copy.deepcopy(a), copy.deepcopy(a)
        b['cleanup']['realisation'] = 'recompute'
        c['path'][0]['iteration'] = []
        schedule['entries'] = [a, b, c]
    text = prefix + 'section schedule\n' + j(schedule) + '\n'
    assert text == (root / filename).read_text()
    assert sha(text) == file_pin
    assert sha(j(schedule)) == schedule_pin
    assert schedule['bindings']['budget'] == [3, 0, 0, 0]
    assert sha(text.split('\nsection schedule\n')[0]) == OH
    assert len(envelope.encode()) == 261 and sha(envelope) == IH
    assert json.loads(text.split('section symbols\n')[1].splitlines()[0])['0']['interface_hash'] == IH
    print(f'R17 {filename}: file {file_pin}, schedule {schedule_pin}; object/interface/closure pins unchanged')
