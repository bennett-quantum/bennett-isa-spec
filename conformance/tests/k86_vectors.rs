//! Independent portable-data checks; this crate does not call the reference
//! model checker or infer a certificate verdict from model completeness.
use bennett_isa_conformance::{
    json::{self, Json},
    ref_hash::sha256,
};
use std::{collections::BTreeSet, fs, path::PathBuf};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/accepted-k86-09")
}
fn load(name: &str) -> Json {
    json::parse(&fs::read_to_string(root().join(name)).unwrap()).unwrap()
}
fn string(v: &Json) -> &str {
    v.as_str().unwrap()
}
fn pair(v: &Json) -> (i128, i128) {
    let p = v.as_arr().unwrap();
    (
        string(&p[0]).parse().unwrap(),
        string(&p[1]).parse().unwrap(),
    )
}

#[test]
fn complete_target_and_schedule_hashes_cover_model_bytes_and_not_source_identity() {
    let manifest = load("vectors.json");
    assert_eq!(
        string(manifest.get("format").unwrap()),
        "bennett.native-model-vectors/1"
    );
    let Json::Obj(files) = manifest.get("files").unwrap() else {
        panic!("files")
    };
    for (name, hash) in files {
        let bytes = fs::read(root().join(name)).unwrap();
        assert_eq!(sha256(&bytes), string(hash), "{name}");
    }
    let target = fs::read(root().join("controlled-x.target.json")).unwrap();
    let legacy = fs::read(root().join("legacy.target.json")).unwrap();
    assert_ne!(sha256(&target), sha256(&legacy));
    for (name, bytes) in [
        ("controlled-x.schedule.json", &target),
        ("legacy.schedule.json", &legacy),
    ] {
        let schedule = load(name);
        assert_eq!(
            string(schedule.at_path("bindings.target_hash").unwrap()),
            sha256(bytes)
        );
        assert_eq!(
            string(schedule.at_path("bindings.closure_hash").unwrap()),
            string(
                manifest
                    .at_path("source_identity.controlled_interface_hash")
                    .unwrap()
            )
        );
        assert_eq!(schedule.get("entries").unwrap().as_arr().unwrap().len(), 2);
    }
    assert_ne!(
        sha256(&fs::read(root().join("controlled-x.schedule.json")).unwrap()),
        sha256(&fs::read(root().join("legacy.schedule.json")).unwrap())
    );
    let mut names = BTreeSet::new();
    for case in manifest.get("cases").unwrap().as_arr().unwrap() {
        assert!(names.insert(string(case.get("name").unwrap())));
        let bytes = fs::read(root().join(string(case.get("target").unwrap()))).unwrap();
        assert_eq!(sha256(&bytes), string(case.get("target_hash").unwrap()));
        assert!(["checked", "unchecked", "reject"].contains(&string(case.get("outcome").unwrap())));
    }
    assert_eq!(names.len(), 17);
    for required in [
        "controlled-x",
        "controlled-cx",
        "not-normalized",
        "context-zero",
        "context-positive",
        "unsupported-version",
        "wrong-control-order",
        "duplicate-operand",
        "unreduced-leaf",
        "binary64-angle",
    ] {
        assert!(names.contains(required), "{required}");
    }
}

#[test]
fn phase_and_accuracy_witnesses_are_derived_by_exact_integer_algebra() {
    let manifest = load("vectors.json");
    let expected = manifest.get("expected").unwrap();
    let controlled = |name: &str| -> Vec<Vec<i128>> {
        let t = load(name);
        let base = t.get("gates").unwrap().as_arr().unwrap()[0]
            .at_path("realisation_model.data.matrix")
            .unwrap()
            .as_arr()
            .unwrap();
        let mut out = vec![vec![0; 4]; 4];
        out[0][0] = 1;
        out[1][1] = 1;
        for (r, row) in base.iter().enumerate() {
            for (c, e) in row.as_arr().unwrap().iter().enumerate() {
                let complex = e.as_arr().unwrap();
                let (n, d) = pair(&complex[0]);
                assert_eq!(d, 1);
                assert_eq!(pair(&complex[1]), (0, 1));
                out[r + 2][c + 2] = n;
            }
        }
        out
    };
    let x = controlled("controlled-x.target.json");
    let minus = controlled("negative-x-bound-two.target.json");
    assert_eq!(
        x,
        vec![
            vec![1, 0, 0, 0],
            vec![0, 1, 0, 0],
            vec![0, 0, 0, 1],
            vec![0, 0, 1, 0]
        ]
    );
    assert_eq!(
        minus,
        vec![
            vec![1, 0, 0, 0],
            vec![0, 1, 0, 0],
            vec![0, 0, 0, -1],
            vec![0, 0, -1, 0]
        ]
    );
    let diff: Vec<Vec<_>> = (0..4)
        .map(|r| (0..4).map(|c| x[r][c] - minus[r][c]).collect())
        .collect();
    assert_eq!(diff.iter().flatten().map(|n| n * n).sum::<i128>(), 8);
    for r in 0..4 {
        for c in 0..4 {
            let gram = (0..4).map(|k| diff[k][r] * diff[k][c]).sum::<i128>();
            assert_eq!(gram, if r == c && r >= 2 { 4 } else { 0 });
        }
    }
    assert_eq!(pair(expected.get("operator_norm_error").unwrap()), (2, 1));
    assert_eq!(
        pair(expected.get("numerical_frobenius_squared").unwrap()),
        (8, 1)
    );
    // 2² < 8 < 3² independently proves the advertised numerical outcomes.
    let claims = expected.get("numerical_claims").unwrap().as_arr().unwrap();
    assert_eq!(claims[0].get("bound"), Some(&Json::Num(2.0)));
    assert_eq!(claims[0].get("accept"), Some(&Json::Bool(false)));
    assert_eq!(claims[1].get("bound"), Some(&Json::Num(3.0)));
    assert_eq!(claims[1].get("accept"), Some(&Json::Bool(true)));
    assert_eq!(
        pair(expected.get("binary64_angle").unwrap()),
        (3602879701896397, 36028797018963968)
    );
    // For uncontrolled X and -X, Choi matrices are equal. Y=I4/8,
    // mu=3/8 gives Y±J pivots1/8 and mu I2-Tr_out Y=I2/8.
    assert_eq!(
        pair(expected.at_path("zero_channel_dual.strict_pivot").unwrap()),
        (1, 8)
    );
}
