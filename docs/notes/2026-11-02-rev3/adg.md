# adg — loaded-device-program state across launches

Full revised proposal for council review; `b-isa-spec-adg`. This replaces
the [first-pass note](../2026-11-02-contract-adg.md) as a review document;
it does not edit the published spec or interpreter. Clause numbers refer
to [ISA 0.9](../../../spec/Bennett-ISA-Spec-0.9.md).

Round 1 accepted the memory/ledger rule (sol and astra); the round-2
bennett-4ht re-check accepted the added device-instance scoping statement.
Muse subsequently blocked the proposal on **adg-M1**, a core §7.3/§10
lifetime omission. This revision retains those accepted decisions and
chooses Muse's recommended program-lifetime device state, with per-run
entropy position and effect tape. It still requires renewed sol/astra
review and a later Muse seat; it is not yet ACCEPTED FOR SPEC.

## Defect and North Star scope

ISA §7.1 defines initial memory, allocation table and bump pointer without
saying whether initialization occurs at load or every run. **§3.3 places
the register ledger at program scope; §3.2 lists it among machine-state
components.** §10 gives a run exclusive ownership only until CLOSED;
§7.4 and T4 restore a run's entry state on eligible reversal. These support
program-lifetime state but do not explicitly settle classical-memory
initialization.

The same omission affects §7.3's initially empty console, clock `c=0`
and block sector map. The original §10 addition's unqualified
“initialization is never repeated merely to start a run” could also
incorrectly carry entropy position `p` into the next run, although §10
copies a new entropy tape at each `bennett_run`. A persistence-reading
executor and a reset-reading executor would still return different clock
and block results on the second entry. This is adg-M1's observable S0
contract defect, not an instruction or allocator change.

Dialect 0.3 draft §§6.5–6.6 requires a persistent qreg table in device
memory and parameter/result identities across entries. North Star §3.1's
register handles and §5's REPL/library device model require this lifetime;
its block-device uses require a disk that survives repeated entries. A
reset on every entry loses the table while registers remain live.
Persistence preserves repeated SQLite/kernel invocations and every demo
rung. No capability is rejected or made inert by this repair. Oracle-local
workspace initialization and cleanup remain governed by §5.2.

## Chosen lifetime rule and alternatives

| State | Established | Later run / reversal |
|---|---|---|
| CC/R memory bytes, allocation table, bump pointer (§7.1) | Once per loaded device program | Inherit preceding CLOSED run's state; eligible reversal restores entry values. |
| Register ledger and live identities (§3.3) | Program instance; entries created by the existing operations | Survive CLOSED unless consumed/freed; ownership is released at CLOSED. Ids remain subject to §3.3's non-reuse rule, including reversal. |
| Console bytes, clock `c`, block sector map (§7.3) | At device-program load; block map from `layout.block_image` under K10, else empty | Persist; effects and their reversal change them; CLOSED does not reset them. |
| Copied entropy tape and position `p` (§§7.3/10) | Each `bennett_run`, with `p=0` | Bound to that run; undo restores that run's position, never a previous run's cursor. |
| Active effect tape, dynamic environments and run history (§§7.3–7.4/10) | Each run | Record that run's effects/history; no inheritance into the next run. Saved journals remain a separately versioned tool interface. |
| Host-minted / device-minted budget tokens (§5.4) | Program / minting run respectively | Host token valid until unload; device token invalid after its run is CLOSED (or revoked by reversal). |

Program-lifetime memory and devices are the recommendation: they match the
ledger, T4's entry-state restoration and the existing persistent-table
consumer. Explicitly resetting *all* devices on each run is Muse's other
possible contract decision, but would require different cross-entry disk
assumptions and would not preserve the selected persistence rule. An
implicit reset when no register is live still changes classical/device
results and has no supporting clause. The recommendation changes none of
§7.1's heap/table limits or `free` semantics.

## Concrete lifecycle witnesses

### Classical memory retained from round 1

Use `ptr_width=16, H=16, A_tab=2, S_cells=0, null_size=1,
heap_base=16, heap_limit_max=16` and one global `g` at base 1, size 1,
image `00`. Run A allocates one byte (address 16), stores 7 in `g`,
completes and is CLOSED. Run B loads `g`, allocates one byte and completes.
The results and full allocation state after B are:

| A disposition before B | B result `(g, allocation)` | Bump | Table `(base,size,live,pinned)` |
|---|---|---|---|
| COMPLETED → CLOSED | `(7,17)` | 18 | `[(1,1,1,1),(16,1,1,0),(17,1,1,0)]` |
| COMPLETED → REVERSED → CLOSED | `(0,16)` | 17 | `[(1,1,1,1),(16,1,1,0)]` |

All heap bytes are zero in this witness; `g` is respectively `07` or
`00`, and every other logical byte is zero. A reset-reading executor would
return `(0,16)` even without reversal, distinguishing the interpretations.
If A also frees 16, B still allocates 17: the dead entry remains and
storage is not reclaimed. After allocations at 16, 17 and 18 across CLOSED
runs, a fourth heap allocation faults with code 2, with bump still 19 and
no table/byte mutation: the global plus three heap entries fill the table.
A FAILED R call whose completed prefix allocated 16 and stored 7 leaves
those mutations in place on close; explicit eligible reversal removes
them. Closing is never implicit rollback.

### Device witness resolving adg-M1

Use one loaded instance, sequential device CC launches whose bodies call
R helpers. The layout has disjoint live globals: `src` at 16, size 512,
all bytes `a5`; `dst` at 528, size 512, zero; `clk` at 1040, size 8,
zero; `ent` at 1048, size 1, zero. Set `ptr_width=16, H=12, A_tab=3,
S_cells=0, null_size=1, heap_base=2048, heap_limit_max=64`.
`block_image` contains sector 0 = 512 zero bytes and sector 1 = 512
`ff` bytes, so sector 2 is initially **absent**.

Run A's R helper executes, in order: `clock.read(clk)`,
`block.write(2,src)`, `console.write(src,1)`, `entropy.read(ent,1)`.
Its copied entropy tape is `[11]` (hex bytes). It writes clock value 0,
leaves `c=1`, creates sector 2 with 512 `a5` bytes, appends console byte
`a5`, consumes its entropy byte and has four effect-tape entries.

Run B's R helper executes `clock.read(clk)`, `block.read(2,dst)` and
`entropy.read(ent,1)`, with its own copied tape `[33]`. It returns
`(load.i64 clk, load.i8 dst, load.i8 ent)`:

| A disposition before B | B result (decimal) | Device state after B | Tape belonging to B |
|---|---|---|---|
| COMPLETED → CLOSED | `(1,165,51)` | `c=2`, console=`a5`, sector 2 present with 512 `a5` bytes | Three entries, beginning with B's clock read of 1; entropy entry is byte `33`. |
| COMPLETED → REVERSED → CLOSED | `(0,0,51)` | `c=1`, console empty, sector 2 absent; B reads all 512 zero bytes | Three entries, beginning with B's clock read of 0; entropy entry is byte `33`. |

In either case B starts with `p=0`, ends with `p=1`, and its tape never
contains A's four entries. Closing A releases its record; reversal first
removes its active effects. Saved journals do not become B's tape. Inspect
all 512 bytes of `dst`, not only the returned first byte. Sectors 0 and 1 remain present
with their initial bytes; reversal must preserve the distinction between
present-zero sector 0 and absent sector 2. A FAILED after the four effects
has the same retained device prefix until reversed. Reversal restores the
modeled console sequence; T4 does **not** retract externally observed output.
Fresh loading restores the initial clock/console/block map and memory.

## Recommended exact spec diff

These are the complete additions to the published 0.9 text, superseding
the first-pass diff. They are not a patch to that unpublished proposal.

Insert after §7.1's Initial state sentence:

```diff
 Initial state: bump = `heap_base`, table = the pre-entries in layout
 order, memory zero except images.
+That state is established once when the program is loaded. These
+persistence rules concern CC/R classical memory, allocation-table entries
+and the bump pointer. These components belong to the loaded program and
+persist across runs. For sequential launches, a new run inherits these
+components from the preceding CLOSED run. Closing a run does not restore
+its entry state. If the preceding run was successfully REVERSED before
+being CLOSED, these classical components have their restored entry values
+(§7.4, §10, T4); quantum-state restoration retains T4's exactness
+hypotheses and error bounds. A FAILED run otherwise leaves its completed
+prefix mutations in place. Register-ledger entries and live register
+identities likewise persist under §3.3; ownership ends at CLOSED, not the
+register lifetime. Unloading and loading a program creates a new program
+instance with initial state; register ids of the old instance are not
+imported into it.
```

Insert immediately after §7.3's device table, before **BLOCK-IMAGE**.
The following paragraph is Muse's requested wording verbatim:

```diff
+Device-model state belongs to the loaded program like classical memory (§7.1): the console byte sequence, the clock counter c and the block sector map are established at load from their initial values (the block map from layout.block_image, K10) and persist across runs, changed only by effect execution and by reversal (§7.4, T4); closing a run does not reinitialise them. The entropy position p is per-run state: it starts at 0 over the entropy tape copied at that bennett_run (§10). The effect tape is per-run.
```

Append to §10's **States** paragraph, after its final §3.3 reference:

```diff
+Persistence does not relax exclusive ownership of touched registers or
+memory while a run is not CLOSED. Separate loaded program instances have
+separate state; the §7.1/§7.3 program-lifetime state is never reinitialised
+merely to start a run. This persistence rule does not prohibit disjoint
+concurrent runs permitted by the existing ownership rules.
```

The §10 sentence deliberately names only program-lifetime state. §7.3
positively defines the separate per-run entropy and effect-tape lifetimes;
the amendment does not inherit dynamic environments, history or oracle
workspace into later runs. The witnesses use sequential CLOSED runs and
invent neither a concurrent memory merge nor a new device synchronization
protocol.

Add after the §15 suite table (a suite G requirement):

```diff
+**Suite G — loaded-program lifetime (K-new).** Execute successive CC
+launches A and B in one loaded device program, each CLOSED before the
+next launch. With ptr_width=16, H=16, A_tab=2, S_cells=0, null_size=1,
+heap_base=16, heap_limit_max=16 and global g at base 1, size 1, image 00,
+let A allocate one byte and store 7 in g; let B return g and a new
+one-byte allocation. Without A reversal B returns (7,17), bump=18 and
+table=[(1,1,1,1),(16,1,1,0),(17,1,1,0)]; with successful A reversal
+before close B returns (0,16), bump=17 and
+table=[(1,1,1,1),(16,1,1,0)], in (base,size,live,pinned) order.
+Check every logical byte and table entry. Closing a FAILED A preserves
+its completed stores and allocations unless explicitly reversed.
+Bytes in the following device vector are hexadecimal. Use live, disjoint
+buffers and an initial block image with sector 0 all zero and sector 1
+all ff (512 bytes each), so
+sector 2 is absent. A reads clock 0, writes 512 a5 bytes to sector 2,
+appends a5 to the console and consumes byte 11 from its copied entropy
+tape. B reads the clock and sector 2 and consumes byte 33 from a new
+one-byte entropy tape. After unreversed A, B reads clock 1 and 512 a5
+bytes; after reversed A, B reads clock 0 and 512 zero bytes, with sector
+2 absent. Check sector presence, console persistence/restoration and
+unchanged sectors 0 and 1. In both cases B starts p=0, reads entropy
+byte 33 and has its own three-entry effect tape without A's entries.
+Repeat with A FAILED after its effects, with and without eligible
+reversal. Fresh load restores the initial program state. Check that
+CLOSED releases register ownership without destroying live identities,
+that host-minted tokens remain valid until unload, and that a device-
+minted token passed after its run is CLOSED returns E_ARG.
```

Add to §0.2, assigning the next available K number when applied:

```text
K-new | §7.1, §7.3, §10, §15 | LOADED-PROGRAM-LIFETIME: CC/R memory,
allocation table, bump pointer and console/clock/block state persist
across runs in one loaded device instance; entropy position and effect
tape are per-run; CLOSED does not reverse. | b-isa-spec-adg | S | –
```

## Hashes, catalog and consumers

Under the **council-authorized in-version 0.9 K repair**, canonical source,
semantic envelope, target and catalog bytes remain unchanged; no hash
re-pin is needed. A full ISA-version bump would change §9.2 envelopes and
require new pins. This note changes neither `ref-0.9.json` nor expansion
recipes, charges, resources or target capabilities; no catalog check is
claimed here. Published 0.8 text is untouched.

A consumer resetting §7.1 memory or console/clock/block on each run must
separate its load and run lifecycles. The plugin's persistent qreg table
needs no redesign. Bindings/certificates replaying a later entry must
record its actual device entry-state/preparation assumptions; one fresh
load does not establish coverage of all histories. Entropy buffers remain
copied at launch; tape access remains associated with the particular run.
Existing `E_ARG` and `E_INVALID_STATE` suffice; no status number is added.

### Accepted bennett-4ht statement (verbatim)

In a mixed fat object, 'loaded program' means a loaded device program instance, not the native host process or entire fat binary. The native host may retain that instance and repeatedly invoke its device CC entries; the specified memory, allocation-table, bump-pointer and register-ledger persistence applies to that instance, with separate state for separate loaded instances. Loading a device program or closing or reversing a device run does not initialize or roll back unrelated native host state. This rule does not require moving native main, argv/envp handling or host-only libc into the device object. Later-entry certificate replay records the actual device entry-state preparation assumptions, without presuming a fresh native process.

That is the adg paragraph accepted in
[the round-2 4ht dossier](../../../../bennett/docs/council/accept-4ht-recheck-2026-11-02/round-2/doc.md).
The new console/clock/block rule has the same device-instance scope;
its per-run tapes cover device R effects, not unrelated native host I/O.
The ordinary native host binary and only marked device modules plus their
linked device closures remain reachable (North Star §§2, 3.3, 4.4).

The [shared fat-object scope](../2026-11-02-rev2/fat-object-scope.md) supplies
the remaining bennett-4ht publication obligations: a versioned
container-to-device-object/root/certificate inventory, native-call to device
entry bindings, budget and opaque-handle lifetime negotiation, and
device-run/tape association with effect-proxy ordering and replay. A general
host-memory-by-reference ABI must also specify address/layout mapping,
range/liveness, aliasing, ownership/synchronization, writeback and reversal.
The current copied-argument/hostcall ABI does not supply those mechanics.
They remain explicit boundary work, never grounds to require whole-binary
lowering or omit mixed execution. Certificates and native comparison retain
the selected materialised device closure as their subject.

## Read-only interpreter witness and reproduction

The public reference `ClassicalMachine` creates initial memory/devices in
its constructor; the CLI's `run_program` is explicitly one-shot. There is
no public API for transferring all program-owned state to a second run.
Thus the following **executed instruction/reversal witness** checks the
device primitives and their observable values, not a multi-launch ABI.
The cross-launch CLOSED transitions, separate tapes while retaining
devices, ownership and token vectors remain explicit suite G obligations.

The harness builds an ISA 0.9 object, computes and checks every interface
hash, and invokes the unmodified library through a job-local Cargo package.
`keep` calls A then executes B's operations in the same run. `undo` first
uncalls A, then executes B. Their one copied entropy tape is `[11,22]`:
B reads `22` without uncall and `11` with uncall, as it must **within one
run**. A fresh B-only machine supplied `[33]` reads `33` at position 0.
This intentionally differs from the two-launch vector's B entropy `33`
and separate three-entry tape. No reset or fake close is inserted into
the reference machine.

Both completed and FAILED A are also executed and whole-run reversed.
Complete snapshot equality checks memory, device state (including sector
presence), ledger, control, dynamic instances and fault/result state; the
initialized block map includes the present-zero sector. It does not inspect
retained audit-history layout or establish cross-run ledger/token lifetimes.
`keep` and `undo` check all 512 read bytes and tape counts, then verify
whole-run restoration too.

```rust
use bennett_isa_ref::{json::Json, ClassicalMachine, ClassicalValue, Object};
use std::collections::BTreeMap;

fn json(s: &str) -> Json { Json::parse(s).unwrap() }

fn object() -> Object {
    let a = "fn r @a () () () () ()\n^entry():\n\
        s = const.i16 16\np = inttoptr s\n\
        k = const.i16 1040\nc = inttoptr k\n\
        e = const.i16 1048\nep = inttoptr e\n\
        sector = const.i64 2\none = const.i16 1\n\
        effect clock.read c\neffect block.write sector p\n\
        effect console.write p one\neffect entropy.read ep one\nret";
    let b = "k = const.i16 1040\nc = inttoptr k\n\
        d = const.i16 528\ndp = inttoptr d\n\
        e = const.i16 1048\nep = inttoptr e\n\
        sector = const.i64 2\none = const.i16 1\n\
        effect clock.read c\neffect block.read sector dp\n\
        effect entropy.read ep one\n\
        cv = load.i64 c\nbv = load.i8 dp\nev = load.i8 ep\nret cv bv ev";
    let mut functions = vec![
        ("a", a.to_string()),
        ("failed", a.replace("@a ", "@failed ").replace("\nret", "\nfault 2")),
        ("b", format!("fn r @b () (cv:i64, bv:i8, ev:i8) () () ()\n^entry():\n{b}")),
    ];
    for (name, prefix) in [
        ("keep", "call @a\n"),
        ("undo", "call @a\nuncall @a () ()\n"),
        ("read", ""),
    ] {
        functions.push((name, format!(
            "fn cc @{name} () (cv:i64, bv:i8, ev:i8) () () ()\n^entry():\n{prefix}(cv, bv, ev) = call @b\nret cv bv ev"
        )));
    }
    let data = [
        ("src", "a5".repeat(512)), ("dst", "00".repeat(512)),
        ("clk", "00".repeat(8)), ("ent", "00".to_string()),
        ("disk", format!("{}{}", "00".repeat(512), "ff".repeat(512))),
    ];
    let mut symbols = BTreeMap::new();
    for (name, body) in &functions {
        let kind = if body.starts_with("fn r ") { "r" } else { "cc" };
        let results = if matches!(*name, "a" | "failed") { "[]" } else { "[\"i64\",\"i8\",\"i8\"]" };
        let sig = json(&format!(
            r#"{{"abi":"0.9","kind":"{kind}","params":[],"results":{results},"tier":"{kind}"}}"#
        ));
        symbols.insert(symbols.len().to_string(), json(&format!(
            r#"{{"name":"{name}","kind":"{kind}","signature":{},"interface_hash":"{}"}}"#,
            sig.canonical(), "0".repeat(64)
        )));
    }
    for (name, hex) in &data {
        symbols.insert(symbols.len().to_string(), json(&format!(
            r#"{{"name":"{name}","kind":"data","signature":{{"abi":"0.9","kind":"data","size":{},"tier":"-"}},"interface_hash":"{}"}}"#,
            hex.len()/2, "0".repeat(64)
        )));
    }
    let tail = r#"
section devices
{"models":{"block":{"version":"0.9"},"clock":{"version":"0.9"},"console":{"version":"0.9"},"entropy":{"version":"0.9"}},"services":{}}

section layout
{"A_tab":3,"H":12,"S_cells":0,"block_image":"disk","globals":{"src":{"base":16,"size":512,"image":"src"},"dst":{"base":528,"size":512,"image":"dst"},"clk":{"base":1040,"size":8,"image":"clk"},"ent":{"base":1048,"size":1,"image":"ent"}},"heap_base":2048,"heap_limit_max":64,"interface_regions":{},"null_size":1,"ptr_width":16}
"#;
    let render = |symbols: BTreeMap<String, Json>| {
        let mut text = format!(
            "bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=16\n\nsection symbols\n{}\n\nsection functions\n",
            Json::Object(symbols).canonical()
        );
        for (_, body) in &functions { text.push_str(&format!("{body}\n")); }
        text.push_str("\nsection data\n");
        for (name, hex) in &data { text.push_str(&format!("{name} {hex}\n")); }
        for line in tail.lines() {
            let line = if line.starts_with('{') { json(line).canonical() } else { line.into() };
            text.push_str(&format!("{line}\n"));
        }
        text
    };
    let parsed = Object::parse(&render(symbols.clone())).unwrap();
    let hashes = parsed.identity().unwrap().interface_hashes;
    for symbol in symbols.values_mut() {
        let name = symbol.get("name").unwrap().string().unwrap().to_string();
        let Json::Object(fields) = symbol else { panic!("symbol is not an object") };
        fields.insert("interface_hash".into(), Json::String(hashes[&name].clone()));
    }
    let object = Object::parse(&render(symbols)).unwrap();
    object.check_hashes().unwrap();
    object
}

fn main() {
    let o = object();
    for (name, fault) in [("a", None), ("failed", Some(2))] {
        let mut m = ClassicalMachine::new(&o, name, &[], &[0x11]).unwrap();
        let entry = m.snapshot();
        let outcome = m.run(100).unwrap();
        assert_eq!(outcome.fault, fault);
        assert!(!m.is_running());
        assert_eq!(m.clock(), 1);
        assert_eq!(m.console(), &[0xa5]);
        assert_eq!(m.effect_tape_len(), 4);
        assert_eq!(&*m.effect_tape()[0].bytes, &[0; 8]);
        assert_eq!(&*m.effect_tape()[1].bytes, &[0xa5; 512]);
        assert_eq!(&*m.effect_tape()[3].bytes, &[0x11]);
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), entry, "complete restoration of {name}");
        println!("{name}: clock=1 console=a5 tape=4; reverse=entry");
    }
    for (name, entropy, clock, byte, ent, tape, console) in [
        ("keep", vec![0x11,0x22], 1, 0xa5, 0x22, 7, vec![0xa5]),
        ("undo", vec![0x11,0x22], 0, 0, 0x11, 3, vec![]),
        ("read", vec![0x33], 0, 0, 0x33, 3, vec![]),
    ] {
        let mut m = ClassicalMachine::new(&o, name, &[], &entropy).unwrap();
        let entry = m.snapshot();
        let out = m.run(100).unwrap();
        assert_eq!(out.fault, None);
        assert!(!m.is_running());
        assert_eq!(&*out.values, &[
            ClassicalValue::Int(64,clock), ClassicalValue::Int(8,byte),
            ClassicalValue::Int(8,ent),
        ]);
        assert_eq!(m.memory_bytes(528,512).unwrap(), vec![byte as u8;512]);
        assert_eq!(m.clock() as u128, clock+1);
        assert_eq!(m.console(), console.as_slice());
        assert_eq!(m.effect_tape_len(), tape);
        assert_eq!(&*m.effect_tape()[tape-1].bytes, &[ent as u8]);
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), entry, "complete restoration of {name}");
        println!("{name}: ({clock},{byte},{ent}) block=512 checked bytes tape={tape}; reverse=entry");
    }
}
```

Run this exact command from any directory. All generated files and Cargo
outputs stay under `.tmp/sol-adg-rev/`; the dependency source is read-only.
There are two Cargo jobs, a 4 GiB per-process address-space bound (in
addition to the runner's 14 GiB scope), no core dumps and a 240-second
timeout. This is one small witness executable, not a repository gate.

```sh
env -u DISPLAY -u WAYLAND_DISPLAY -u DBUS_SESSION_BUS_ADDRESS -u BROWSER \
  TMPDIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-adg-rev/tmp \
  CARGO_TARGET_DIR=/home/tobias/Projects/bennett-workspace/bennett-isa-spec/.tmp/sol-adg-rev/target \
  python3 - <<'PY'
import os, pathlib, resource, subprocess
repo = pathlib.Path('/home/tobias/Projects/bennett-workspace/bennett-isa-spec')
job = repo / '.tmp/sol-adg-rev'
(job / 'tmp').mkdir(parents=True, exist_ok=True)
crate = job / 'witness'
(crate / 'src').mkdir(parents=True, exist_ok=True)
note = (repo / 'docs/notes/2026-11-02-rev3/adg.md').read_text()
(crate / 'src/main.rs').write_text(note.split('```rust\n', 1)[1].split('\n```', 1)[0] + '\n')
(crate / 'Cargo.toml').write_text('''[package]
name = "adg-device-witness"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
bennett-isa-ref = { path = "../../../reference-interpreter" }
[profile.dev]
debug = 0
incremental = false
codegen-units = 4
''')
resource.setrlimit(resource.RLIMIT_AS, (4 * 1024**3,) * 2)
resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
subprocess.run(['timeout', '240s', 'cargo', 'run', '--offline',
                '--manifest-path', str(crate / 'Cargo.toml'), '-j2', '--quiet'], check=True)
PY
```

Expected stdout (the verification result below records whether reproduced):

```text
a: clock=1 console=a5 tape=4; reverse=entry
failed: clock=1 console=a5 tape=4; reverse=entry
keep: (1,165,34) block=512 checked bytes tape=7; reverse=entry
undo: (0,0,17) block=512 checked bytes tape=3; reverse=entry
read: (0,0,51) block=512 checked bytes tape=3; reverse=entry
```

Verification result: the exact bounded command above exited **0** and
reproduced all five stdout lines. Every assertion passed, including full
512-byte comparisons and complete entry-snapshot restoration for all five
cases. The reference interpreter and conformance sources were not edited.
No full gate or actual repeated-launch ABI test was run.

The bounded document self-check also passed: Muse's §7.3 paragraph and the
accepted 4ht statement match verbatim, §10 has the scoped lifetime phrase,
the citation is corrected, all four spec insertions apply at their stated
anchors in an in-memory dry run, fences balance and all six links resolve.

## Additional conformance obligations

At publication, suite G must exercise the **actual** two-launch sequences
above, comparing complete memory/table/bump, ledger and device state,
token validity and run-associated tapes. An executed `qreg.from_id` of
another open run's register produces terminal fault code 4; supplying that
register as a conflicting host launch argument returns `E_INVALID_STATE`
before effects. After A is CLOSED, its stored live id can be recovered by
`qreg.from_id` in B. A device-minted token passed after A is CLOSED returns
`E_ARG`; a host-minted token remains valid until unload. Separate loaded
instances have separate state. A native host invoking a retained device
instance must also check that device reversal leaves unrelated host state
unchanged. These vectors are requirements, not tests claimed to have run
through the current one-shot interpreter.

## Muse blocking items and resolution

| Review item | Resolution in this proposal | Evidence / remaining validation |
|---|---|---|
| **adg-M1 (core), §7.3 lifetime missing** | Add Muse's paragraph verbatim after the device table: console/clock/block established at load and persistent; `p=0` and effect tape per run. | Concrete two-launch vector: B clock 1 / sector a5 without A reversal; clock 0 / absent sector 2 and 512 zero bytes with reversal. Real interpreter effects and complete reversal are checked by the bounded witness; actual multi-launch API conformance remains to implement. |
| **adg-M1 (core), §10 initialization too broad** | Replace the original unqualified phrase with “the §7.1/§7.3 program-lifetime state is never reinitialised merely to start a run”. | B's new one-byte tape `[33]` always yields `33` at `p=0`, and B owns exactly three effects. The within-run harness explicitly retains its cursor, distinguishing run boundaries from calls. |
| **adg-M1 required conformance vectors** | Add the clock/block vectors to §15 suite G, including reversal, FAILED-prefix retention, present-zero versus absent sectors, console and independent entropy/tapes. | Exact full byte patterns and state transitions are specified above; no lifecycle implementation pass is inferred from a scalar assignment or one-shot execution. |
| **adg-4HT-1, previously resolved** | Preserve the accepted paragraph verbatim in “Hashes, catalog and consumers”; extend the same instance scope to the new devices. | Native host binary, marked device modules, later-entry replay and unrelated host state remain separated as in the accepted re-check and shared scope. |
| **Round-1 citation correction** | Attribute program scope to §3.3; §3.2 only lists machine state. | Corrected in the defect paragraph and the complete spec diff. |

Muse's `suggestions` array is empty; its report's S1/S2 items address
z4c, 5u3 or checker-diagnostics, not adg. There is no unaddressed adg S1.
The source is
[out-muse.json](../../../../bennett/docs/council/accept-muse-batch-2026-11-02/round-1/out-muse.json)
and its [report](../../../../bennett/docs/council/accept-muse-batch-2026-11-02/round-1/report.md).
Round-1 optional refinements are also retained here: explicit T4 quantum
qualifications, full scalar table states, free/table-capacity boundary,
token/ownership distinctions and the in-version hash condition.

## Release and after acceptance

Recommend a **0.9 K wording repair to §§7.1, 7.3 and 10**, with the §15
lifecycle vectors. The core change must return to sol and astra and then a
later Muse seat, as the convenor requires. Once accepted and authorized,
the spec editor assigns the K number, applies the complete additions plus
the accepted 4ht statement, and regenerates the relevant indexes/diffs.
Reference/VM owners then implement and test repeated-launch lifetimes;
the dialect updates its cross-reference. Keep `b-isa-spec-adg` open until
the contract decision and its required follow-through land. `bennett-4ht`
continues to track the mixed-launch boundary. No full gates, spec edits,
interpreter changes, commits or remote operations are part of this job.
