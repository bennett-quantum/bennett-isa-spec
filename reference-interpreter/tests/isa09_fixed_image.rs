//! Every expected tuple/verdict is derived in the portable vector generator,
//! independently of the interpreter. K85 preserves exact phase and full tuples.
mod support;
use bennett_isa_ref::{
    json::Json, ClaimVerdict, Object, QuantumBindings, QuantumRegion, QuantumState,
};
use std::{collections::BTreeMap, path::PathBuf};

fn fields(value: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(fields) = value else {
        panic!("object")
    };
    fields
}

fn root(object: &Object) -> &str {
    object
        .section("certificate")
        .unwrap()
        .get("bindings")
        .unwrap()
        .get("closure_hash")
        .unwrap()
        .string()
        .unwrap()
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../conformance/fixtures/accepted-k85-09/{name}.bisa"
    )))
    .unwrap()
}

fn mutate(source: &str, change: impl FnOnce(&mut Json)) -> String {
    let (base, cert) = source.split_once("section certificate\n").unwrap();
    let mut cert = Json::parse(cert.trim()).unwrap();
    let record = fields(fields(&mut cert).get_mut("regions").unwrap())
        .values_mut()
        .next()
        .unwrap();
    change(record);
    format!("{base}section certificate\n{}\n", cert.canonical())
}

fn claim(record: &mut Json) -> &mut Json {
    let Json::Array(claims) = fields(record).get_mut("claims").unwrap() else {
        panic!("claims")
    };
    &mut claims[0]
}

#[test]
fn published_fixed_image_vectors_replay_with_exact_coverage_and_bindings() {
    let manifest = Json::parse(include_str!(
        "../../conformance/fixtures/accepted-k85-09/k85.json"
    ))
    .unwrap();
    for vector in manifest.get("cases").unwrap().array().unwrap() {
        let name = vector.get("name").unwrap().string().unwrap();
        let text = fixture(name);
        assert_eq!(
            bennett_isa_ref::sha256_hex(text.as_bytes()),
            vector.get("sha256").unwrap().string().unwrap(),
            "{name}"
        );
        let outcome = vector.get("outcome").unwrap().string().unwrap();
        let parsed = Object::parse(&text);
        if outcome == "reject_admission" {
            let error = parsed.unwrap_err().to_string();
            assert!(error.contains("§12.2"), "{name}: {error}");
            continue;
        }
        let object = parsed.unwrap_or_else(|e| panic!("{name}: {e}"));
        object.check_hashes().unwrap();
        if outcome == "reject_replay" {
            assert!(
                object.verify_claim(root(&object), 0).is_err(),
                "{name}: direct checker"
            );
            assert!(
                object.verify_certificate().is_err(),
                "{name}: certificate checker"
            );
            continue;
        }
        let n = vector.get("elements").unwrap().uint().unwrap();
        let expected = match outcome {
            "proof" => ClaimVerdict::Proof,
            "proof_relative_to_image" => ClaimVerdict::ProofRelativeToImage,
            "evidence" => ClaimVerdict::Evidence { elements: n },
            _ => panic!("unexpected outcome {outcome}"),
        };
        assert_eq!(
            object.verify_claim(root(&object), 0).unwrap(),
            expected,
            "{name}"
        );
        let report = object.verify_certificate().unwrap().claims.remove(0);
        assert_eq!(report.verdict, expected, "{name}");
        assert_eq!(report.closure_hash, root(&object));
        assert_eq!(report.index, 0);
        assert_eq!(report.context_key, support::json("[]"));
        let coverage = report.coverage.unwrap();
        assert_eq!(coverage.elements, n, "{name}");
        assert_eq!(
            coverage.full_domain_elements,
            vector
                .get("full_domain_elements")
                .unwrap()
                .string()
                .unwrap(),
            "{name}"
        );
        assert_eq!(
            coverage.covered_fraction,
            vector.get("covered_fraction").unwrap().string().unwrap(),
            "{name}"
        );
        if name == "report-order" {
            // Producer order is aux,db. Signature mem slots are db,aux.
            assert_eq!(
                coverage
                    .fixed_regions
                    .iter()
                    .map(|b| (b.group, b.slot))
                    .collect::<Vec<_>>(),
                vec![(2, 0), (2, 1)]
            );
            assert_eq!(
                coverage.fixed_regions[0].sha256,
                bennett_isa_ref::sha256_hex(&[1, 2])
            );
            assert_eq!(
                coverage.fixed_regions[1].sha256,
                bennett_isa_ref::sha256_hex(&[1])
            );
            assert_eq!(
                coverage
                    .fixed_regions
                    .iter()
                    .map(|b| b.id.as_str())
                    .collect::<Vec<_>>(),
                vec!["mem:0", "mem:1"]
            );
        }
    }
}

#[test]
fn requested_labels_are_exclusive_for_full_and_image_relative_proofs() {
    for (fixture_name, computed) in [
        ("full-d-x", "proof"),
        ("image-0102", "proof_relative_to_image"),
        ("omit-a", "evidence"),
        ("sampled-full-d", "evidence"),
    ] {
        for label in ["proof", "proof_relative_to_image", "evidence"] {
            let text = mutate(&fixture(fixture_name), |record| {
                fields(claim(record)).insert("label".into(), Json::String(label.into()));
            });
            let object = Object::parse(&text).unwrap();
            let accepts = label == "evidence" || label == computed;
            assert_eq!(
                object.verify_claim(root(&object), 0).is_ok(),
                accepts,
                "{fixture_name}: {label}"
            );
            assert_eq!(
                object.verify_certificate().is_ok(),
                accepts,
                "{fixture_name}: {label}"
            );
        }
    }
}

#[test]
fn structural_full_d_theorem_cannot_satisfy_an_image_relative_request() {
    let text = mutate(&fixture("full-d-x"), |record| {
        *claim(record) = support::json(
            r#"{"kind":"structural","rule":"adjoint_table","subclaims":[],"witness":{},"invocation":{"args":[],"budget":["0","0","0","0"]},"label":"proof_relative_to_image"}"#,
        );
    });
    let object = Object::parse(&text).unwrap();
    // K94 establishes a full-D adjoint theorem. K85's relative request is
    // exclusive; stronger/different theorem categories cannot stand in for it.
    assert!(object.verify_claim(root(&object), 0).is_err());
    assert!(object.verify_certificate().is_err());
}

#[test]
fn fixed_image_proof_does_not_discharge_an_unrestricted_borrow_frame() {
    let text = mutate(&fixture("controlled-dirty"), |record| {
        fields(fields(record).get_mut("verification").unwrap())
            .insert("borrow_frames".into(), support::json("[0]"));
    });
    let object = Object::parse(&text).unwrap();
    assert_eq!(
        object.verify_claim(root(&object), 0).unwrap(),
        ClaimVerdict::ProofRelativeToImage
    );
    let error = object.verify_certificate().unwrap_err().to_string();
    assert!(error.contains("proof over D"), "{error}");
}

#[test]
fn full_tuples_include_every_fixed_cell_and_preserved_bundle() {
    for field in ["bundles", "regions"] {
        let text = mutate(&fixture("image-0102"), |record| {
            let Json::Array(rows) = fields(fields(claim(record)).get_mut("expected").unwrap())
                .get_mut("table")
                .unwrap()
            else {
                panic!("rows")
            };
            let after = fields(fields(&mut rows[0]).get_mut("after").unwrap());
            let Json::Array(values) = after.get_mut(field).unwrap() else {
                panic!("values")
            };
            if field == "regions" {
                let Json::Array(cells) = &mut values[0] else {
                    panic!("cells")
                };
                cells[0] = Json::String("0".into());
            } else {
                values[1] = Json::String("1".into());
            }
        });
        let object = Object::parse(&text).unwrap();
        let error = object
            .verify_claim(root(&object), 0)
            .unwrap_err()
            .to_string();
        assert!(error.contains("full tuple mismatch"), "{field}: {error}");
    }
}

#[test]
fn certificate_image_metadata_never_changes_executable_identities() {
    let a = Object::parse(&fixture("image-0102")).unwrap();
    let b = Object::parse(&fixture("image-0201")).unwrap();
    let a_id = a.identity().unwrap();
    let b_id = b.identity().unwrap();
    assert_eq!(a_id.object_hash, b_id.object_hash);
    assert_eq!(a_id.interface_hashes, b_id.interface_hashes);
    assert_eq!(a_id.closure_hashes, b_id.closure_hashes);
    // Old closed 0.8 readers must reject the 0.9 fields/label explicitly.
    let legacy = fixture("image-0102")
        .replace("=0.9", "=0.8")
        .replace("\"0.9\"", "\"0.8\"");
    assert!(Object::parse(&legacy).is_err());
}

#[test]
fn controlled_fixed_map_restores_dirty_and_arbitrary_entangled_references() {
    let object = Object::parse(&fixture("external-reference")).unwrap();
    assert_eq!(
        object.verify_claim(root(&object), 0).unwrap(),
        ClaimVerdict::ProofRelativeToImage
    );
    let mut q = QuantumState::new();
    let t = q.allocate(1).unwrap();
    let f = q.allocate(4).unwrap();
    let a = q.allocate(1).unwrap();
    let cells = vec![q.allocate(1).unwrap(), q.allocate(1).unwrap()];
    let dirty = q.allocate(1).unwrap();
    let control = q.allocate(1).unwrap();
    let references = [
        q.allocate(1).unwrap(),
        q.allocate(1).unwrap(),
        q.allocate(1).unwrap(),
    ];
    q.gate("x", &cells[0], None, &[]).unwrap();
    for bit in [f[0], f[2]] {
        q.gate("x", &[bit], None, &[]).unwrap();
    }
    for (wire, reference) in [&a, &dirty, &control].into_iter().zip(&references) {
        q.gate("h", wire, None, &[]).unwrap();
        q.gate("cx", &[wire[0], reference[0]], None, &[]).unwrap();
    }
    let mut bindings = QuantumBindings::default();
    for (name, wires) in [("t", &t), ("f", &f), ("a", &a)] {
        bindings.bundles.insert(name.into(), wires.clone());
    }
    bindings.regions.insert(
        "db".into(),
        QuantumRegion {
            address_width: 1,
            cell_width: 1,
            cells: cells.clone(),
        },
    );
    bindings.borrowed.push(dirty.clone());
    object
        .execute_action("lookup", &mut q, &bindings, false, &control)
        .unwrap();
    // Independently derived inverse of t ^= control & !a. The other
    // systems, including the three untouched references, must factor back.
    q.gate("x", &a, None, &[]).unwrap();
    q.gate("ccx", &[control[0], a[0], t[0]], None, &[]).unwrap();
    q.gate("x", &a, None, &[]).unwrap();
    for (wire, reference) in [&a, &dirty, &control].into_iter().zip(&references).rev() {
        q.gate("cx", &[wire[0], reference[0]], None, &[]).unwrap();
        q.gate("h", wire, None, &[]).unwrap();
    }
    q.gate("x", &cells[0], None, &[]).unwrap();
    for bit in [f[0], f[2]] {
        q.gate("x", &[bit], None, &[]).unwrap();
    }
    for wires in [&t, &f, &a, &cells[0], &cells[1], &dirty, &control]
        .into_iter()
        .chain(references.iter())
    {
        // release checks exact-zero amplitudes, not a binary64 tolerance.
        q.release(wires).unwrap();
    }
    assert_eq!(q.wire_count(), 0);
}
