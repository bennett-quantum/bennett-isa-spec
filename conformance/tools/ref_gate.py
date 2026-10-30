"""Headless offline family gate; run with python3 -I. No interpreter source reads."""
import os
from pathlib import Path
import subprocess
r=Path(__file__).resolve().parents[2]
tmp=r/'.tmp';tmp.mkdir(exist_ok=True)
env=dict(os.environ,TMPDIR=str(tmp))
for k in ['DISPLAY','WAYLAND_DISPLAY','DBUS_SESSION_BUS_ADDRESS','BROWSER']:env.pop(k,None)
cmds=[['cargo','fmt','--check','--manifest-path',str(r/'conformance/Cargo.toml')],['cargo','clippy','--offline','--all-targets','--manifest-path',str(r/'conformance/Cargo.toml'),'--','-D','warnings'],['cargo','test','--offline','--manifest-path',str(r/'conformance/Cargo.toml')],['cargo','run','--offline','--manifest-path',str(r/'conformance/Cargo.toml'),'--','--bin',str(r/'reference-interpreter/target/debug/bennett-isa-ref')]]
for i,c in enumerate(cmds):
    p=subprocess.run(c,cwd=r,env=env,capture_output=True,text=True)
    text=p.stdout+p.stderr;(tmp/f'ref-gate-{i}.log').write_text(text)
    print(i,p.returncode,text[-1800:] if p.returncode not in (0,2) else text[-450:],flush=True)
    if p.returncode!=(2 if i==3 else 0):raise SystemExit(p.returncode or 1)
    if i==3:assert '| all | 445 | 0 |' in text,text[-800:]
