from pathlib import Path
import resource, subprocess
p = Path(__file__).resolve().parent
root = p.parents[1]
def guard():
    resource.setrlimit(resource.RLIMIT_AS, (192*1024*1024,)*2)
    resource.setrlimit(resource.RLIMIT_CORE, (0,0))
for name in ['false-cnot-proof.bisa', 'false-cnot-renamed-control.bisa', 'false-cnot-full-control.bisa']:
    r = subprocess.run([str(p/'report'), str(p/name)], text=True, capture_output=True, preexec_fn=guard)
    print(name, 'exit', r.returncode, '\n'+r.stdout+r.stderr)
    assert r.returncode == 0
for name in ['a', 'identity']:
    r = subprocess.run([str(root/'reference-interpreter/target/debug/bennett-isa-ref'), 'run-action', str(p/'false-cnot-proof.bisa'), name, '--mut', '1,0'], text=True, capture_output=True, preexec_fn=guard)
    print(name, 'at t=1,u=0', 'exit', r.returncode, '\n'+r.stdout+r.stderr)
    assert r.returncode == 0
