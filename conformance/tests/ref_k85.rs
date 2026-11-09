//! Portable K85 readers deliberately admit fixed_regions and the third
//! successful category; old closed readers must upgrade explicitly.
use bennett_isa_conformance::{
    json::{self, Json},
    ref_hash::sha256,
};
use std::{collections::BTreeSet, path::PathBuf};

const VECTORS: &str = include_str!("../fixtures/accepted-k85-09/k85.json");
const RESTRICTED: &str = include_str!("../fixtures/accepted-k85-09/restricted-map.json");

fn text(value: &Json) -> &str {
    value.as_str().unwrap()
}

fn keys(value: &Json) -> BTreeSet<&str> {
    let Json::Obj(fields) = value else {
        panic!("object")
    };
    fields.keys().map(String::as_str).collect()
}

#[test]
fn portable_fixed_image_reader_preserves_closed_schema_and_independent_hashes() {
    let vectors = json::parse(VECTORS).unwrap();
    assert_eq!(
        text(vectors.get("format").unwrap()),
        "bennett.fixed-images.v1"
    );
    let mut names = BTreeSet::new();
    let mut image_relations = 0;
    for vector in vectors.get("cases").unwrap().as_arr().unwrap() {
        let name = text(vector.get("name").unwrap());
        assert!(names.insert(name), "duplicate vector {name}");
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/accepted-k85-09")
            .join(text(vector.get("object").unwrap()));
        let source = std::fs::read_to_string(path).unwrap();
        assert_eq!(
            sha256(source.as_bytes()),
            text(vector.get("sha256").unwrap()),
            "{name}"
        );
        let certificate =
            json::parse(source.split_once("section certificate\n").unwrap().1.trim()).unwrap();
        let Json::Obj(regions) = certificate.get("regions").unwrap() else {
            panic!("regions")
        };
        let claim = &regions
            .values()
            .next()
            .unwrap()
            .get("claims")
            .unwrap()
            .as_arr()
            .unwrap()[0];
        let domain = claim.get("domain").unwrap();
        let allowed = BTreeSet::from(["bundles", "borrowed", "regions", "args", "fixed_regions"]);
        assert!(
            keys(domain).is_subset(&allowed),
            "unknown domain field in {name}"
        );
        if let Some(label) = claim.get("label") {
            assert!(["proof", "proof_relative_to_image", "evidence"].contains(&text(label)));
        }
        if name == "image-0102" || name == "image-0201" {
            image_relations += 1;
            let image = &domain.get("fixed_regions").unwrap().as_arr().unwrap()[0];
            assert_eq!(
                keys(image),
                BTreeSet::from(["name", "A", "W", "image", "sha256"])
            );
            let bytes = match text(image.get("image").unwrap()) {
                "0102" => [1, 2],
                "0201" => [2, 1],
                other => panic!("unexpected image {other}"),
            };
            assert_eq!(sha256(&bytes), text(image.get("sha256").unwrap()));
            let rows = claim
                .get("expected")
                .unwrap()
                .get("table")
                .unwrap()
                .as_arr()
                .unwrap();
            assert_eq!(rows.len(), 64);
            for (ordinal, row) in rows.iter().enumerate() {
                // t outermost, f next, a fastest: ordinal=32*t+2*f+a.
                let t = ordinal / 32;
                let f = (ordinal / 2) % 16;
                let a = ordinal % 2;
                for (side, value) in [("before", t), ("after", t ^ (bytes[a] as usize & 1))] {
                    let tuple = row.get(side).unwrap();
                    let bundles = tuple.get("bundles").unwrap().as_arr().unwrap();
                    assert_eq!(
                        bundles.iter().map(text).collect::<Vec<_>>(),
                        [value.to_string(), f.to_string(), a.to_string()]
                    );
                    let cells = &tuple.get("regions").unwrap().as_arr().unwrap()[0];
                    assert_eq!(
                        cells.as_arr().unwrap().iter().map(text).collect::<Vec<_>>(),
                        bytes.map(|n| n.to_string())
                    );
                    assert!(tuple.get("borrowed").unwrap().as_arr().unwrap().is_empty());
                }
            }
            assert_eq!(text(vector.get("covered_fraction").unwrap()), "1/65536");
            assert_eq!(text(vector.get("full_domain_elements").unwrap()), "4194304");
        }
    }
    assert_eq!(image_relations, 2);
    for required in [
        "swapped-hash",
        "stale-hash",
        "hex-text-hash",
        "omit-a",
        "omit-f",
        "omit-t",
        "sampled-full-d",
        "mut-region",
        "nonzero-padding",
        "duplicate-image",
        "controlled-dirty",
        "external-reference",
    ] {
        assert!(names.contains(required), "missing vector {required}");
    }
}

fn matrix(value: &Json) -> Vec<Vec<i64>> {
    value
        .as_arr()
        .unwrap()
        .iter()
        .map(|row| {
            row.as_arr()
                .unwrap()
                .iter()
                .map(|n| {
                    let Json::Num(n) = n else { panic!("integer") };
                    assert_eq!(n.fract(), 0.0);
                    *n as i64
                })
                .collect()
        })
        .collect()
}

#[test]
fn restricted_z_identity_map_has_reduced_input_and_complete_output() {
    let vector = json::parse(RESTRICTED).unwrap();
    let u = matrix(vector.get("U").unwrap());
    let v = matrix(vector.get("V").unwrap());
    let j = matrix(vector.get("J").unwrap());
    assert_eq!(u, vec![vec![1, 0], vec![0, -1]]);
    assert_eq!(v, vec![vec![1, 0], vec![0, 1]]);
    assert_eq!(j, vec![vec![1], vec![0]]);
    assert_eq!(vector.get("input_dimension"), Some(&Json::Num(1.0)));
    assert_eq!(vector.get("output_dimension"), Some(&Json::Num(2.0)));
    // Exact integer algebra: no interpreter state or numerical output is read.
    let difference = (0..2)
        .map(|r| (0..2).map(|c| u[r][c] - v[r][c]).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let restricted = difference
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(c, x)| x * j[c][0])
                .sum::<i64>()
        })
        .collect::<Vec<_>>();
    assert_eq!(restricted, vec![0, 0]);
    assert_eq!(
        vector.get("restricted_frobenius_squared"),
        Some(&Json::Num(0.0))
    );
    assert_eq!(difference.iter().flatten().map(|x| x * x).sum::<i64>(), 4);
    assert_eq!(
        vector.get("unrestricted_frobenius_squared"),
        Some(&Json::Num(4.0))
    );
    // Reduced input dimension one times complete output dimension two gives
    // a 2x2 Choi difference, not 1x1 and not unrestricted 4x4.
    assert_eq!(
        matrix(vector.get("restricted_choi_difference").unwrap()),
        vec![vec![0, 0], vec![0, 0]]
    );
    let dual = vector.get("strict_dual").unwrap();
    let y = dual.get("Y").unwrap().as_arr().unwrap();
    assert_eq!(y.len(), 2);
    assert_eq!(
        y[0].as_arr().unwrap().iter().map(text).collect::<Vec<_>>(),
        vec!["0.125", "0"]
    );
    assert_eq!(
        y[1].as_arr().unwrap().iter().map(text).collect::<Vec<_>>(),
        vec!["0", "0.125"]
    );
    assert_eq!(text(dual.get("mu").unwrap()), "0.375");
    assert_eq!(text(dual.get("bound").unwrap()), "0.375");
    // Exact dyadic arithmetic in units of 1/8 for this diagonal witness.
    let eighths = |s: &str| match s {
        "0" => 0,
        "0.125" => 1,
        "0.375" => 3,
        _ => panic!("unexpected witness endpoint {s}"),
    };
    let a = eighths(text(&y[0].as_arr().unwrap()[0]));
    let b = eighths(text(&y[1].as_arr().unwrap()[1]));
    let mu = eighths(text(dual.get("mu").unwrap()));
    assert!(a > 0 && b > 0);
    assert_eq!(mu - (a + b), 1);

    let phase = vector.get("phase_sensitive").unwrap();
    let phase_u = matrix(phase.get("U").unwrap());
    let phase_v = matrix(phase.get("V").unwrap());
    assert_eq!(phase_u, vec![vec![-1, 0], vec![0, -1]]);
    assert_eq!(phase_v, v);
    let column = |m: &[Vec<i64>]| {
        (0..2)
            .map(|r| (0..2).map(|c| m[r][c] * j[c][0]).sum::<i64>())
            .collect::<Vec<_>>()
    };
    let prepared_u = column(&phase_u);
    let prepared_v = column(&phase_v);
    let phase_norm_squared = prepared_u
        .iter()
        .zip(&prepared_v)
        .map(|(u, v)| (u - v) * (u - v))
        .sum::<i64>();
    assert_eq!(phase_norm_squared, 4);
    assert_eq!(
        phase.get("restricted_frobenius_squared"),
        Some(&Json::Num(phase_norm_squared as f64))
    );
    let choi = (0..2)
        .map(|r| {
            (0..2)
                .map(|c| prepared_u[r] * prepared_u[c] - prepared_v[r] * prepared_v[c])
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        choi,
        matrix(phase.get("uncontrolled_choi_difference").unwrap())
    );
    assert_eq!(choi, vec![vec![0, 0], vec![0, 0]]);
}
