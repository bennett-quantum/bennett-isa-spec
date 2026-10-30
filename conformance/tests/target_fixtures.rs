//! Independent §8.1 fixture obligations, not values learned from an executor.
use bennett_isa_conformance::{
    cases_dir,
    json::{self, Json},
};
use std::{fs, process::Command};

#[test]
fn canonical_target_bytes_and_hash() {
    let root = cases_dir().join("E");
    for entry in fs::read_dir(&root).unwrap() {
        let path = entry.unwrap().path().join("target.json");
        if !path.exists() {
            continue;
        }
        let bytes = fs::read(&path).unwrap();
        assert_eq!(
            bytes.last(),
            Some(&b'}'),
            "§8.1 JCS: no trailing LF: {}",
            path.display()
        );
        // All these fixtures contain only ASCII keys, integer numbers and booleans.
        let script = "import json,sys; b=open(sys.argv[1],'rb').read(); assert b == json.dumps(json.loads(b),sort_keys=True,separators=(',',':')).encode()";
        assert!(Command::new("python3")
            .args(["-c", script, path.to_str().unwrap()])
            .status()
            .unwrap()
            .success());
    }
    let path = root.join("E-jcs-target-hash/target.json");
    let hash = Command::new("sha256sum").arg(&path).output().unwrap();
    assert!(hash.status.success());
    let actual = String::from_utf8(hash.stdout).unwrap();
    let doc = json::parse(&fs::read_to_string(root.join("E-jcs-target-hash/case.json")).unwrap())
        .unwrap();
    assert_eq!(
        doc.at_path("expected_outcome.target_hash")
            .and_then(Json::as_str),
        Some(actual.split_whitespace().next().unwrap())
    );
}

#[test]
fn exact_and_forbidden_epsilon_witnesses_are_distinct() {
    let root = cases_dir().join("E");
    let load = |name: &str| {
        json::parse(&fs::read_to_string(root.join(name).join("target.json")).unwrap()).unwrap()
    };
    let bad = load("E-epsilon-zero-entry");
    let gate = &bad.get("gates").unwrap().as_arr().unwrap()[0];
    assert_eq!(
        gate.at_path("cost.time.var").and_then(Json::as_str),
        Some("eps"),
        "§8.1 negative must actually reference eps"
    );
    assert_eq!(gate.get("epsilon"), Some(&Json::Num(0.0)));
    let good = load("E-epsilon-zero-exact");
    let gate = &good.get("gates").unwrap().as_arr().unwrap()[0];
    assert_eq!(gate.at_path("cost.time"), Some(&Json::Num(1.0)));
    assert_eq!(gate.get("epsilon"), Some(&Json::Num(0.0)));
    for (name, outcome) in [
        ("E-epsilon-zero-entry", "reject"),
        ("E-epsilon-zero-exact", "accept"),
    ] {
        let doc =
            json::parse(&fs::read_to_string(root.join(name).join("case.json")).unwrap()).unwrap();
        assert_eq!(
            doc.at_path("expected_outcome.load").and_then(Json::as_str),
            Some(outcome)
        );
    }
    let canonical = fs::read(root.join("E-target-staged-probe/target.json")).unwrap();
    let noncanonical = fs::read(root.join("E-target-staged-probe/noncanonical.json")).unwrap();
    assert_eq!(
        noncanonical,
        [canonical, vec![b'\n']].concat(),
        "negative JCS witness adds exactly one LF"
    );
}
