use bennett_isa_ref::{json::Json, sha256_hex, Object};
use std::{collections::BTreeMap, path::PathBuf, process::Command};

const BASE: &str = include_str!("../../conformance/fixtures/accepted-k84-09/baseline.bisa");

fn fields(value: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(fields) = value else {
        panic!("object")
    };
    fields
}

fn certificate(text: &str) -> Json {
    Json::parse(text.split_once("section certificate\n").unwrap().1.trim()).unwrap()
}

fn claim(cert: &mut Json) -> &mut Json {
    let region = fields(fields(cert).get_mut("regions").unwrap())
        .values_mut()
        .next()
        .unwrap();
    let Json::Array(claims) = fields(region).get_mut("claims").unwrap() else {
        panic!("claims")
    };
    &mut claims[0]
}

#[test]
fn k84_vectors_reject_only_the_named_mutation_under_its_normative_clause() {
    let directory =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../conformance/fixtures/accepted-k84-09");
    let manifest = Json::parse(include_str!(
        "../../conformance/fixtures/accepted-k84-09/k84.json"
    ))
    .unwrap();
    for case in manifest.get("cases").unwrap().array().unwrap() {
        let name = case.get("name").unwrap().string().unwrap();
        let path = directory.join(case.get("object").unwrap().string().unwrap());
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            sha256_hex(text.as_bytes()),
            case.get("sha256").unwrap().string().unwrap()
        );
        // Compare the complete certificate, not only the changed claim. This
        // locks unchanged identity, invocation, expected table and all records.
        let mut expected = certificate(BASE);
        match name {
            "baseline" => {}
            "missing-expected" => {
                fields(claim(&mut expected)).remove("expected").unwrap();
            }
            "duplicate-component" => {
                let domain = fields(claim(&mut expected)).get_mut("domain").unwrap();
                let Json::Array(bundles) = fields(domain).get_mut("bundles").unwrap() else {
                    panic!("bundles")
                };
                bundles.push(bundles[0].clone());
            }
            _ => panic!("unexpected vector {name}"),
        }
        assert_eq!(certificate(&text), expected);
        assert_eq!(
            text.split_once("section certificate\n").unwrap().0,
            BASE.split_once("section certificate\n").unwrap().0
        );
        let result = Object::parse(&text);
        let output = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
            .arg("check")
            .arg(path)
            .output()
            .unwrap();
        if name == "baseline" {
            result.unwrap().check_hashes().unwrap();
            assert_eq!(output.status.code(), Some(0));
            assert!(output.stderr.is_empty());
        } else {
            let clause = case
                .get("normative_clause")
                .unwrap()
                .string()
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap();
            assert!(result.unwrap_err().to_string().contains(clause));
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8_lossy(&output.stderr).contains(clause));
        }
    }
}

#[test]
fn k84_baseline_executes_the_full_not_table_and_its_inverse() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../conformance/fixtures/accepted-k84-09/baseline.bisa");
    for input in 0..2 {
        for command in ["run-action", "reverse"] {
            let output = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
                .arg(command)
                .arg(&path)
                .args(["flip", "--mut", &input.to_string()])
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            let result = Json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
            assert_eq!(
                result.get("mut").unwrap().array().unwrap()[0]
                    .uint()
                    .unwrap(),
                if command == "reverse" {
                    input
                } else {
                    input ^ 1
                }
            );
        }
    }
}
