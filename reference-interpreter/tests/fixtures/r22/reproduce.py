from pathlib import Path
import subprocess,resource,json,hashlib
D=Path(__file__).resolve().parent
R=Path('/home/tobias/Projects/bennett-workspace/bennett-isa-spec-review/reference-interpreter')
def limit():
 resource.setrlimit(resource.RLIMIT_AS,(128*1024*1024,)*2)
 resource.setrlimit(resource.RLIMIT_CORE,(0,0))
assert hashlib.sha256((D/'hostcall-64MiB.envelope').read_bytes()).hexdigest()=='655e72211b66a9e4e3f8dcaff74c5203da6ec1d834fc43570833f0ab1387a577'
assert hashlib.sha256((D/'hostcall-64MiB.bisa').read_bytes()).hexdigest()=='df7f95dd72496eb446c39319d5ef1e6287ebed8fdd6b9a7f92eba31ba1431071'
records=[]
for command in [[str(R/'target/debug/bennett-isa-ref'),'check',str(D/'hostcall-64MiB.bisa')],[str(D/'host-probe'),str(D/'hostcall-64MiB.bisa')],[str(R/'target/debug/bennett-isa-ref'),'run-cc',str(D/'hostcall-64MiB.bisa'),'f']]:
 p=subprocess.run(command,preexec_fn=limit,capture_output=True,text=True,timeout=60)
 record=dict(command=command,exit=p.returncode,stdout=p.stdout,stderr=p.stderr);records.append(record);print(json.dumps(record))
(D/'reproduction.json').write_text(json.dumps(records,indent=2))
