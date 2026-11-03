use bennett_isa_conformance::{
    json as source_json,
    ref_catalog::Machine,
    ref_language::{invert_trace, permutation},
};
use bennett_isa_ref::{json::Json, sha256_hex, ClaimVerdict, Object};
use std::{collections::BTreeSet, path::PathBuf};
const MANIFEST: &str = include_str!("../../conformance/fixtures/accepted-k91-09/k91.json");
const MANIFEST_SHA: &str = "72b68db2dfbb041fc259f95d3669ff29438bfec5cdcb356fc4b1c713b7dd53fa";
fn directory() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../conformance/fixtures/accepted-k91-09")
}
fn manifest() -> Json {
    assert_eq!(sha256_hex(MANIFEST.as_bytes()), MANIFEST_SHA);
    Json::parse(MANIFEST).unwrap()
}
#[test]
fn published_k91_objects_pin_all_source_identities_and_binding_hashes() {
    let m = manifest();
    let source = m.get("source").unwrap();
    let bytes =
        std::fs::read(directory().join(source.get("object").unwrap().string().unwrap())).unwrap();
    assert_eq!(
        sha256_hex(&bytes),
        source.get("sha256").unwrap().string().unwrap()
    );
    let base = Object::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();
    base.check_hashes().unwrap();
    let identity = base.identity().unwrap();
    assert_eq!(
        identity.object_hash,
        m.get("object_hash").unwrap().string().unwrap()
    );
    let mut schedule_hashes = BTreeSet::new();
    let mut certificate_hashes = BTreeSet::new();
    for row in m.get("fixtures").unwrap().array().unwrap() {
        let bytes =
            std::fs::read(directory().join(row.get("object").unwrap().string().unwrap())).unwrap();
        assert_eq!(
            sha256_hex(&bytes),
            row.get("sha256").unwrap().string().unwrap()
        );
        let parsed = Object::parse(std::str::from_utf8(&bytes).unwrap());
        if row.get("accepted").unwrap() == &Json::Bool(false) {
            assert!(parsed.is_err());
            continue;
        }
        let object = parsed.unwrap();
        object.check_hashes().unwrap();
        let actual = object.identity().unwrap();
        assert_eq!(actual.object_hash, identity.object_hash);
        assert_eq!(actual.interface_hashes, identity.interface_hashes);
        assert_eq!(actual.closure_hashes, identity.closure_hashes);
    }
    for row in m.get("partitions").unwrap().array().unwrap() {
        let object = Object::parse(
            &std::fs::read_to_string(
                directory().join(row.get("object").unwrap().string().unwrap()),
            )
            .unwrap(),
        )
        .unwrap();
        let schedule = object.section("schedule").unwrap();
        let certificate = object.section("certificate").unwrap();
        let sh = sha256_hex(schedule.canonical().as_bytes());
        let ch = sha256_hex(certificate.canonical().as_bytes());
        assert_eq!(sh, row.get("schedule_hash").unwrap().string().unwrap());
        assert_eq!(ch, row.get("certificate_hash").unwrap().string().unwrap());
        assert_eq!(
            schedule
                .get("bindings")
                .unwrap()
                .get("params_hash")
                .unwrap(),
            m.get("params_hash").unwrap()
        );
        schedule_hashes.insert(sh);
        certificate_hashes.insert(ch);
    }
    assert_eq!(schedule_hashes.len(), 2);
    assert_eq!(certificate_hashes.len(), 2);
}
#[test]
fn published_k91_source_carriers_and_claims_cover_both_complete_identity_tables() {
    let m = manifest();
    for row in m.get("partitions").unwrap().array().unwrap() {
        let object = Object::parse(
            &std::fs::read_to_string(
                directory().join(row.get("object").unwrap().string().unwrap()),
            )
            .unwrap(),
        )
        .unwrap();
        let closure = object.identity().unwrap().closure_hashes["pair"]
            .clone()
            .unwrap();
        assert_eq!(
            object.verify_claim(&closure, 0).unwrap(),
            ClaimVerdict::Proof
        );
        let labels = row
            .get("token_aliases")
            .unwrap()
            .array()
            .unwrap()
            .iter()
            .map(|v| v.uint().unwrap() as u64)
            .collect::<Vec<_>>();
        let args =
            source_json::parse(r#"["t","f","x",["1","0","0","0"],["1","0","0","0"]]"#).unwrap();
        let bundles = [("t", 1), ("f", 4), ("x", 1)];
        let leaves = object
            .expand_action_source_with_token_aliases(
                "pair",
                args.as_arr().unwrap(),
                &bundles,
                false,
                &labels,
            )
            .unwrap();
        let mut actual = BTreeSet::new();
        for leaf in &leaves {
            let s = leaf.source.as_ref().unwrap();
            if s.path.is_empty() && s.terminal.symbol == closure && leaf.ordinal == 0 {
                actual.insert((
                    s.terminal.point.clone(),
                    s.terminal.iteration.clone(),
                    s.terminal.instance.clone(),
                    0,
                ));
            }
        }
        let expected = row
            .get("carriers")
            .unwrap()
            .array()
            .unwrap()
            .iter()
            .map(|c| {
                assert_eq!(c.get("symbol").unwrap().string().unwrap(), closure);
                (
                    c.get("point").unwrap().string().unwrap().to_string(),
                    vec![],
                    vec![],
                    c.get("generated").unwrap().uint().unwrap() as usize,
                )
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(actual, expected);
        let mut inverse = leaves.clone();
        invert_trace(&mut inverse).unwrap();
        for (a, b) in inverse.iter().zip(leaves.iter().rev()) {
            assert_eq!(a.source, b.source);
            assert_eq!(a.invocation, b.invocation);
            assert_eq!(a.ordinal, b.ordinal);
        }
        for trace in [&leaves, &inverse] {
            let machine = Machine::compile(&permutation(trace).unwrap(), &bundles).unwrap();
            let mut state = machine.packed_state();
            for lane in 0..64 {
                machine.set_lane(&mut state, "t", (lane >> 5) as u64, lane);
                machine.set_lane(&mut state, "f", ((lane >> 1) & 15) as u64, lane);
                machine.set_lane(&mut state, "x", (lane & 1) as u64, lane);
            }
            let before = state.clone();
            machine.run_packed(&mut state, u64::MAX, false).unwrap();
            assert_eq!(state, before, "complete identity, clean private wires");
            machine.run_packed(&mut state, u64::MAX, true).unwrap();
            assert_eq!(state, before);
        }
    }
}
