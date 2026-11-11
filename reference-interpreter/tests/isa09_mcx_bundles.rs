mod support;
use bennett_isa_conformance::{
    ref_amplitudes::{column, Complex as ExactComplex},
    ref_language::Leaf,
};
use bennett_isa_ref::{json::Json, ref_library::Value, Object, QuantumBindings, QuantumState};
use std::path::PathBuf;

const MANIFEST: &str = include_str!("../../conformance/fixtures/accepted-k93-09/k93.json");
fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../conformance/fixtures/accepted-k93-09")
            .join(name),
    )
    .unwrap()
}
fn strings(value: &Json) -> Vec<String> {
    value
        .array()
        .unwrap()
        .iter()
        .map(|v| v.string().unwrap().into())
        .collect()
}
fn integers(value: &Json) -> Vec<usize> {
    value
        .array()
        .unwrap()
        .iter()
        .map(|v| v.uint().unwrap() as usize)
        .collect()
}
fn coordinates(values: &[usize], widths: &[usize]) -> Vec<bool> {
    values
        .iter()
        .zip(widths)
        .flat_map(|(v, w)| (0..*w).map(move |i| v >> i & 1 == 1))
        .collect()
}
fn basis(values: &[usize], widths: &[usize]) -> usize {
    coordinates(values, widths)
        .iter()
        .enumerate()
        .fold(0, |v, (i, b)| v | (usize::from(*b) << i))
}
fn prepared(
    names: &[String],
    widths: &[usize],
    values: &[usize],
) -> (QuantumState, QuantumBindings) {
    let mut q = QuantumState::new();
    let mut bindings = QuantumBindings::default();
    for ((name, width), value) in names.iter().zip(widths).zip(values) {
        let wires = q.allocate(*width).unwrap();
        for (i, wire) in wires.iter().enumerate() {
            if value >> i & 1 == 1 {
                q.gate("x", &[*wire], None, &[]).unwrap();
            }
        }
        bindings.bundles.insert(name.clone(), wires);
    }
    (q, bindings)
}
fn leaves(object: &Object, names: &[String], widths: &[usize], adjoint: bool) -> Vec<Leaf> {
    let args = names
        .iter()
        .map(|n| Value::Str(n.clone()))
        .collect::<Vec<_>>();
    let bundles = names
        .iter()
        .zip(widths)
        .map(|(n, w)| (n.as_str(), *w))
        .collect::<Vec<_>>();
    object
        .expand_action_source("a", &args, &bundles, adjoint)
        .unwrap()
}

#[test]
fn every_published_basis_column_and_adjoint_has_exact_positive_unit_amplitude() {
    let manifest = Json::parse(MANIFEST).unwrap();
    for vector in manifest.get("vectors").unwrap().array().unwrap() {
        let object = support::rehash(&fixture(vector.get("object").unwrap().string().unwrap()));
        let names = strings(vector.get("names").unwrap());
        let widths = integers(vector.get("widths").unwrap());
        for row in vector.get("rows").unwrap().array().unwrap() {
            let before = integers(row.get("before").unwrap());
            let after = integers(row.get("after").unwrap());
            for adjoint in [false, true] {
                let (input, output) = if adjoint {
                    (&after, &before)
                } else {
                    (&before, &after)
                };
                let (mut q, bindings) = prepared(&names, &widths, input);
                object
                    .execute_action("a", &mut q, &bindings, adjoint, &[])
                    .unwrap();
                assert_eq!(
                    q.exact_support(),
                    vec![(coordinates(output, &widths), (1.0, 0.0))],
                    "{} adj={adjoint}",
                    vector.canonical()
                );
            }
        }
    }
}

#[test]
fn source_adapter_columns_match_independent_formula_vectors_in_both_directions() {
    let manifest = Json::parse(MANIFEST).unwrap();
    for vector in manifest.get("vectors").unwrap().array().unwrap() {
        let object = support::rehash(&fixture(vector.get("object").unwrap().string().unwrap()));
        let names = strings(vector.get("names").unwrap());
        let widths = integers(vector.get("widths").unwrap());
        let wires = names
            .iter()
            .zip(&widths)
            .flat_map(|(n, w)| (0..*w).map(move |i| (n.clone(), i)))
            .collect::<Vec<_>>();
        for adjoint in [false, true] {
            let expanded = leaves(&object, &names, &widths, adjoint);
            for row in vector.get("rows").unwrap().array().unwrap() {
                let before = integers(row.get("before").unwrap());
                let after = integers(row.get("after").unwrap());
                let (input, output) = if adjoint {
                    (&after, &before)
                } else {
                    (&before, &after)
                };
                let mut expected = vec![ExactComplex::default(); 1 << wires.len()];
                expected[basis(output, &widths)] = ExactComplex::one();
                assert_eq!(
                    column(&expanded, &wires, basis(input, &widths)).unwrap(),
                    expected,
                    "{} adj={adjoint}",
                    vector.canonical()
                );
            }
        }
    }
}

#[test]
fn mixed_written_order_and_active_predicate_have_k3_four_operands_and_c4() {
    let names = ["t", "r", "s", "b"].map(String::from);
    let object = support::rehash(&fixture("predicate.bisa"));
    let expanded = leaves(&object, &names, &[1, 1, 1, 2], false);
    assert_eq!(expanded.len(), 1);
    assert_eq!(expanded[0].gate, "mcx");
    assert_eq!(
        expanded[0].wires,
        vec![
            ("s".into(), 0),
            ("b".into(), 0),
            ("b".into(), 1),
            ("t".into(), 0)
        ]
    );
    assert_eq!(expanded[0].controls, vec![("r".into(), 0)]);
    assert_eq!(expanded[0].wires.len() - 1, 3);
    assert_eq!(expanded[0].wires.len() - 1 + expanded[0].controls.len(), 4);
    assert_eq!(
        expanded[0].source.as_ref().unwrap().terminal.point,
        "entry:1"
    );
    let names = ["t", "s", "b"].map(String::from);
    let first = support::rehash(&fixture("mixed-sb.bisa"));
    let second = support::rehash(&fixture("mixed-bs.bisa"));
    let a = leaves(&first, &names, &[1, 1, 2], false);
    let b = leaves(&second, &names, &[1, 1, 2], false);
    assert_eq!(
        a[0].wires,
        vec![
            ("s".into(), 0),
            ("b".into(), 0),
            ("b".into(), 1),
            ("t".into(), 0)
        ]
    );
    assert_eq!(
        b[0].wires,
        vec![
            ("b".into(), 0),
            ("b".into(), 1),
            ("s".into(), 0),
            ("t".into(), 0)
        ]
    );
    assert_ne!(
        first.identity().unwrap().object_hash,
        second.identity().unwrap().object_hash
    );
    assert_ne!(
        first.identity().unwrap().closure_hashes,
        second.identity().unwrap().closure_hashes
    );
    assert_eq!(
        a[0].source.as_ref().unwrap().terminal.point,
        b[0].source.as_ref().unwrap().terminal.point
    );
}

#[test]
fn original_witness_retains_published_byte_identities() {
    let object = Object::parse(&fixture("witness.bisa")).unwrap();
    object.check_hashes().unwrap();
    let id = object.identity().unwrap();
    assert_eq!(
        id.object_hash,
        "db3d3f92a79077074901449d83532f1a0d82ef1621bd833adb7ad14d1aedd467"
    );
    assert_eq!(
        id.closure_hashes["a"].as_deref(),
        Some("e38d5da64f9a0ae89a66243bcbe5a321cb807a5bbabf83952b63d3465dfab346")
    );
    let library = object.bound_ref_library().unwrap();
    // These source/closure bytes are unchanged; an omitted library binding
    // now selects the published r1 default rather than the frozen original.
    assert_eq!(
        library.hash(),
        "0be00e64de17699516a9b5bb5daf308ee0c59af7e39b6bf934bad5c628f48e7a"
    );
}

#[test]
fn external_reference_coherence_preserves_both_exact_prepared_amplitudes() {
    let object = Object::parse(&fixture("witness.bisa")).unwrap();
    let mut q = QuantumState::new();
    let wires = q.allocate(5).unwrap(); // p,q,c0,c1,R
    q.gate("h", &[wires[4]], None, &[]).unwrap();
    q.gate("cx", &[wires[4], wires[2]], None, &[]).unwrap();
    q.gate("cx", &[wires[4], wires[3]], None, &[]).unwrap();
    let before = q.clone();
    // §4.4 H and the §15 witness give 1/sqrt(2) independently of execution.
    let amplitude = (std::f64::consts::FRAC_1_SQRT_2, 0.0);
    assert_eq!(
        q.exact_support(),
        vec![
            (vec![false; 5], amplitude),
            (vec![false, false, true, true, true], amplitude)
        ]
    );
    let mut bindings = QuantumBindings::default();
    bindings.bundles.insert("p".into(), vec![wires[0]]);
    bindings.bundles.insert("q".into(), vec![wires[1]]);
    bindings.bundles.insert("c".into(), wires[2..4].to_vec());
    object
        .execute_action("a", &mut q, &bindings, false, &[])
        .unwrap();
    assert_eq!(
        q.exact_support(),
        vec![
            (vec![false; 5], amplitude),
            (vec![true, false, true, true, true], amplitude)
        ]
    );
    object
        .execute_action("a", &mut q, &bindings, true, &[])
        .unwrap();
    assert_eq!(q, before);
}

#[test]
fn repeated_controls_overlap_and_non_scalar_or_dead_targets_reject_as_operands() {
    for name in [
        "repeated-bundle",
        "repeated-scalar",
        "target-overlap",
        "scalar-target-overlap",
        "multiwire-target",
        "dead-target",
        "target-out-of-range",
    ] {
        let error = Object::parse(&fixture(&format!("{name}.bisa"))).expect_err(name);
        assert!(error.to_string().contains("§3.3"), "{name}: {error}");
    }
}

#[test]
fn resolved_aliases_reject_before_any_gate_executes() {
    let object = support::rehash(&fixture("mixed-sb.bisa"));
    let names = ["t", "s", "b"].map(String::from);
    let (mut q, mut bindings) = prepared(&names, &[1, 1, 2], &[0, 1, 3]);
    bindings
        .bundles
        .insert("s".into(), vec![bindings.bundles["b"][0]]);
    let before = q.clone();
    let error = object
        .execute_action("a", &mut q, &bindings, false, &[])
        .unwrap_err();
    assert!(error.to_string().contains("§3.3"));
    assert_eq!(q, before);
}

#[test]
fn k78_matches_complete_control_sets_after_bundle_expansion() {
    let object = Object::parse(&fixture("causal-same-controls.bisa")).unwrap();
    object.check_hashes().unwrap();
    // c=0 follows from two matching resolved flips. The subsequent frame
    // may touch arbitrary d only if K78 supplied that exact entry fact.
    let error = Object::parse(&fixture("causal-different-controls.bisa"))
        .expect_err("different effective controls cannot match");
    assert!(error.to_string().contains("§5.6"), "{error}");
}

#[test]
fn rechecked_certificate_proves_all_controls_and_rejects_stale_first_bit_relation() {
    let closure = "e38d5da64f9a0ae89a66243bcbe5a321cb807a5bbabf83952b63d3465dfab346";
    let corrected = Object::parse(&fixture("witness-certified.bisa")).unwrap();
    corrected.check_hashes().unwrap();
    assert_eq!(
        corrected.verify_claim(closure, 0).unwrap(),
        bennett_isa_ref::ClaimVerdict::Proof
    );
    let stale = Object::parse(&fixture("witness-firstbit-certificate.bisa")).unwrap();
    stale.check_hashes().unwrap(); // identical source hashes do not prove the table
    let error = stale.verify_claim(closure, 0).unwrap_err();
    assert!(error.to_string().contains("full tuple mismatch"), "{error}");
    assert_eq!(
        corrected.identity().unwrap().object_hash,
        stale.identity().unwrap().object_hash
    );
}

#[test]
fn cached_callee_transfer_remaps_complete_bundle_for_each_actual_binding() {
    let root = r#"{"abi":"0.9","args":[],"bounds":[],"in":[2,2],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let child = r#"{"abi":"0.9","args":[],"bounds":[],"in":[2],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let body = "action @a (t:wire mut) (b:wire[2] in, c:wire[2] in) () () ()\n^entry():\nd = qborrow any 1\naction_call @g (d) (b) () ()\naction_call @g (d) (b) () ()\nqreturn d";
    let source = support::object(
        &[
            ("a", root, body),
            (
                "g",
                child,
                "perm_action @g (t:wire mut) (b:wire[2] in) () () ()\n^entry():\nmcx (b) t",
            ),
        ],
        "{}",
        "",
    )
    .replace("=0.8", "=0.9");
    support::rehash(&source).check_hashes().unwrap();
    // The callee is cached on its first use. A different second actual must
    // retain both c bits, rather than reusing the first call's b mapping.
    let wrong = source.replacen(
        "action_call @g (d) (b) () ()\nqreturn d",
        "action_call @g (d) (c) () ()\nqreturn d",
        1,
    );
    let error = Object::parse(&wrong).expect_err("independent conjunctions cannot restore d");
    assert!(error.to_string().contains("§5.6"), "{error}");
}

#[test]
fn isa08_scalar_admission_stays_unchanged() {
    let source = fixture("width-2.bisa")
        .replace("=0.9", "=0.8")
        .replace("\"0.9\"", "\"0.8\"");
    assert!(Object::parse(&source)
        .unwrap_err()
        .to_string()
        .contains("§3.3"));
}
