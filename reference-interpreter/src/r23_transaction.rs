//! Stage-indexed transactionality regression family (R23).
use crate::json::Json;
use crate::{allocation_test as inject, *};
use std::collections::BTreeMap;
pub fn json(s: &str) -> Json {
    Json::parse(s).unwrap()
}
pub fn object(functions: &[(&str, &str, &str)], imports: &str, extra: &str) -> String {
    let symbols = functions
        .iter()
        .enumerate()
        .map(|(i, (name, signature, _))| {
            let sig = json(signature);
            let value = Json::Object(BTreeMap::from([
                ("name".into(), Json::String((*name).into())),
                ("kind".into(), sig.get("kind").unwrap().clone()),
                ("signature".into(), sig),
                ("interface_hash".into(), Json::String("0".repeat(64))),
            ]));
            (i.to_string(), value)
        })
        .collect();
    let mut text=format!("bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n{}\n",Json::Object(symbols).canonical());
    if imports != "{}" {
        text.push_str(&format!(
            "\nsection imports\n{}\n",
            json(imports).canonical()
        ));
    }
    text.push_str("\nsection functions\n");
    for (_, _, body) in functions {
        text.push_str(body.trim_end_matches('\n'));
        text.push('\n');
    }
    text.push_str(extra);
    text
}
pub fn rehash(text: &str) -> Object {
    let parsed = Object::parse(text).unwrap();
    let hashes = parsed.identity().unwrap().interface_hashes;
    let mut lines: Vec<_> = text.lines().map(str::to_string).collect();
    let i = lines.iter().position(|s| s == "section symbols").unwrap() + 1;
    let Json::Object(mut symbols) = json(&lines[i]) else {
        unreachable!()
    };
    for symbol in symbols.values_mut() {
        let name = symbol.get("name").unwrap().string().unwrap().to_string();
        let Json::Object(value) = symbol else {
            unreachable!()
        };
        value.insert("interface_hash".into(), Json::String(hashes[&name].clone()));
    }
    lines[i] = Json::Object(symbols).canonical();
    Object::parse(&(lines.join("\n") + "\n")).unwrap()
}

fn check_step(o: &Object, entry: &str, prefix: usize, observer: bool) -> usize {
    let setup = || {
        let mut m = ClassicalMachine::new(o, entry, &[], &[3; 512]).unwrap();
        m.push_measurement_draw(0.9).unwrap();
        for _ in 0..prefix {
            m.step_once().unwrap().expect("prefix step");
        }
        m
    };
    let execute = |m: &mut ClassicalMachine| {
        m.step_once_observed(|_, before, after| {
            if observer {
                let delta = before.delta(after)?;
                let digest = after.memory_digest()?;
                Json::String(format!(
                    "{digest}:{}:{}",
                    delta.registers.len(),
                    delta.memory.len()
                ))
                .try_canonical()?;
            }
            Ok(())
        })
    };
    let mut good = setup();
    let entry_state = good.snapshot();
    let frames = good.outcome().steps;
    inject::start(usize::MAX);
    let point = execute(&mut good).unwrap().unwrap().0;
    let stages = inject::finish();
    assert!(!stages.is_empty());
    let expected = good.snapshot();
    if let Some(width) = point.instruction.op.strip_prefix("store.i") {
        let n = width.parse::<usize>().unwrap() / 8;
        let mut bytes = vec![0; n];
        bytes[0] = if point.instruction.operands[1] == "a" {
            2
        } else {
            1
        };
        assert_eq!(good.memory_bytes(1, n).unwrap(), bytes);
    }
    if let Some(width) = point.instruction.op.strip_prefix("load.i") {
        let delta = entry_state.delta(&expected).unwrap();
        assert_eq!(
            delta.registers.last().unwrap().post,
            Some(ClassicalValue::Int(width.parse().unwrap(), 1))
        );
    }
    assert_eq!(good.outcome().fault, None);
    assert_eq!(good.outcome().steps, frames + 1);
    for fail_at in 0..stages.len() {
        let mut m = setup();
        let before = m.snapshot();
        assert_eq!(before, entry_state);
        let tape = m.effect_tape().to_vec();
        let transcript = m.host_transcript().unwrap();
        for _ in 0..2 {
            inject::start(fail_at);
            let e = execute(&mut m).unwrap_err();
            let reached = inject::finish();
            assert_eq!(e.kind(), ErrorKind::HostResource);
            assert_eq!(reached, stages[..=fail_at]);
            assert_eq!(
                m.snapshot(),
                before,
                "{} / stage {fail_at} {}",
                point.instruction.op,
                stages[fail_at]
            );
            assert_eq!(m.outcome().steps, frames);
            assert_eq!(m.effect_tape(), tape);
            assert_eq!(m.host_transcript().unwrap(), transcript);
        }
        let retry = execute(&mut m).unwrap().unwrap().0;
        assert_eq!(retry, point);
        assert_eq!(m.snapshot(), expected);
        assert_eq!(m.outcome().steps, frames + 1);
        if entry != "measurement" {
            if m.outcome().request.is_some() {
                m.reply(&[]).unwrap();
            }
            m.reverse().unwrap();
        }
    }
    eprintln!(
        "R23 {entry}/{} observer={observer}: {} stages {:?}",
        point.instruction.op,
        stages.len(),
        stages
    );
    stages.len()
}
#[test]
fn transactionality_instruction_stages() {
    let signature = r#"{"abi":"0.8","kind":"cc","params":[],"results":[],"tier":"cc"}"#;
    let tail = r#"
section devices
{"models":{"block":{"version":"0.8"},"clock":{"version":"0.8"},"console":{"version":"0.8"},"entropy":{"version":"0.8"}},"services":{"org.bennett.test":{"version":"1"}}}

section layout
{"A_tab":4,"H":32,"S_cells":1,"globals":{},"heap_base":1,"heap_limit_max":4096,"interface_regions":{},"null_size":1,"ptr_width":32}
"#;
    let base = "n = const.i32 512\np = alloc n\nv = const.i8 1\nstore.i8 p v\n";
    let mut cases: Vec<(String, usize)> = Vec::new();
    for w in (8..=128).step_by(8) {
        cases.push((format!("{base}a = const.i{w} 2\nstore.i{w} p a\nret\n"), 5));
        cases.push((format!("{base}a = load.i{w} p\nret\n"), 4));
    }
    for op in [
        "a = alloc n",
        "free p",
        "r = hostcall org.bennett.test p n",
        "effect console.write p n",
        "effect clock.read p",
        "effect entropy.read p n",
        "z = const.i32 0\neffect block.read z p",
        "z = const.i32 0\neffect block.write z p",
    ] {
        cases.push((
            format!("{base}{op}\nret\n"),
            4 + usize::from(op.starts_with("z =")),
        ));
    }
    let mut count = 0;
    let mut stages = 0;
    for (body, prefix) in cases {
        let tier = if body.contains("effect ") { "r" } else { "cc" };
        let function = format!("fn {tier} @f () () () () ()\n^entry():\n{body}");
        let sig = signature.replace("cc", tier);
        let o = rehash(&object(&[("f", &sig, &function)], "{}", tail));
        o.check_hashes().unwrap();
        stages += check_step(&o, "f", prefix, false);
        count += 1;
    }
    let q = Object::parse(include_str!("../tests/fixtures/registers.bisa")).unwrap();
    // Every row of allocation/views/measurement is a separate preparation case,
    // including free, slice, concat, action transfer and consumed-handle output.
    for name in ["allocation", "views", "measurement"] {
        let mut m = ClassicalMachine::new(&q, name, &[], &[]).unwrap();
        let mut prefix = 0;
        while m.is_running() {
            stages += check_step(&q, name, prefix, true);
            count += 1;
            m.step_once().unwrap();
            prefix += 1;
        }
    }
    let f = "fn cc @f () () () () ()\n^entry():\na = call @g\nuncall @g () (a)\nret";
    let g="fn cc @g () (r:i32) () () ()\n^entry():\nn = const.i32 512\np = alloc n\nv = const.i8 1\nstore.i8 p v\nret n";
    let gs = r#"{"abi":"0.8","kind":"cc","params":[],"results":["i32"],"tier":"cc"}"#;
    let nested = rehash(&object(&[("f", signature, f), ("g", gs, g)], "{}", tail));
    let mut m = ClassicalMachine::new(&nested, "f", &[], &[]).unwrap();
    let mut prefix = 0;
    while m.is_running() {
        stages += check_step(&nested, "f", prefix, true);
        count += 1;
        m.step_once().unwrap();
        prefix += 1;
    }
    eprintln!("R23 instruction family: {count} rows / {stages} denied-stage cases; two denials + successful retry per stage");
}
#[test]
fn transactionality_reply_stages() {
    let o = rehash(
        &include_str!("../tests/fixtures/r23/codex-store-retry.bisa").replace("1048576", "8"),
    );
    let setup = || {
        let mut m = ClassicalMachine::new(&o, "f", &[], &[]).unwrap();
        m.run(3).unwrap();
        m
    };
    let execute = |m: &mut ClassicalMachine| {
        m.reply_observed(&[1; 8], |before, after| {
            before.delta(after)?;
            after.memory_digest()?;
            Json::String("reply".into()).try_canonical()?;
            Ok(())
        })
    };
    let mut good = setup();
    inject::start(usize::MAX);
    execute(&mut good).unwrap();
    let stages = inject::finish();
    let expected = good.snapshot();
    for fail_at in 0..stages.len() {
        let mut m = setup();
        let before = m.snapshot();
        let transcript = m.host_transcript().unwrap();
        for _ in 0..2 {
            inject::start(fail_at);
            assert_eq!(execute(&mut m).unwrap_err().kind(), ErrorKind::HostResource);
            assert_eq!(inject::finish(), stages[..=fail_at]);
            assert_eq!(m.snapshot(), before);
            assert_eq!(m.outcome().steps, 3);
            assert_eq!(m.host_transcript().unwrap(), transcript);
        }
        execute(&mut m).unwrap();
        assert_eq!(m.snapshot(), expected);
        assert_eq!(m.outcome().steps, 3);
        assert_eq!(m.run(5).unwrap().values[0], ClassicalValue::Int(64, 2));
        m.reverse().unwrap();
    }
    eprintln!(
        "R23 reply + observer: {} denied-stage cases {:?}",
        stages.len(),
        stages
    );
}
#[test]
fn transactionality_certificate_stages() {
    let text = include_str!("../tests/fixtures/r22/false-cnot-proof.bisa");
    let o = Object::parse(text).unwrap();
    let original = o.identity().unwrap();
    inject::start(usize::MAX);
    let expected = o.verify_certificate().unwrap();
    let stages = inject::finish();
    assert!(!stages.is_empty());
    for fail_at in 0..stages.len() {
        for _ in 0..2 {
            inject::start(fail_at);
            assert_eq!(
                o.verify_certificate().unwrap_err().kind(),
                ErrorKind::HostResource
            );
            assert_eq!(inject::finish(), stages[..=fail_at]);
        }
        assert_eq!(o.identity().unwrap().envelopes, original.envelopes);
        assert_eq!(
            format!("{:?}", o.verify_certificate().unwrap()),
            format!("{expected:?}")
        );
    }
    eprintln!(
        "R23 certificate replay: {} denied-stage cases {:?}",
        stages.len(),
        stages
    );
}

#[test]
fn transactionality_trace_stages() {
    let o = rehash(
        &include_str!("../tests/fixtures/r23/codex-store-retry.bisa").replace("1048576", "8"),
    );
    assert_eq!(check_step(&o, "f", 0, true), 7);
}
