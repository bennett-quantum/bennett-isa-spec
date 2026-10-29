use bennett_isa_ref::{sha256_hex, Object};

#[test]
fn fips_180_4_sha256_vectors() {
    for (input, expected) in [
        ("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        ("abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
        ("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"),
        ("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu", "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"),
    ] { assert_eq!(sha256_hex(input.as_bytes()), expected); }
    assert_eq!(
        sha256_hex(&vec![b'a'; 1_000_000]),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn all_four_published_hashes_and_printed_envelopes() {
    let spec = include_str!("../../spec/Bennett-ISA-Spec.md");
    let mut hashes = std::collections::BTreeMap::new();
    let mut envelopes = Vec::new();
    for fixture in [
        include_str!("fixtures/inc8.bisa"),
        include_str!("fixtures/sum_db.bisa"),
    ] {
        let object = Object::parse(fixture).unwrap();
        let identity = object.identity().unwrap();
        object.check_hashes().unwrap();
        for (name, hash) in identity.interface_hashes {
            assert_eq!(
                identity.closure_hashes[&name].as_deref(),
                Some(hash.as_str())
            );
            hashes.insert(name, hash);
        }
        envelopes.extend(identity.envelopes.into_values());
    }
    for (name, expected) in [
        (
            "inc8",
            "8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93",
        ),
        (
            "zero",
            "7002ed5ea010907a0ae99065663f22fb7d18ca1d6cb171fed57e2594bd9367e6",
        ),
        (
            "sum",
            "74f110bbafde68a4785025e7e571ef871ddd83a09465ab9c1604f653a0cfcd92",
        ),
        (
            "sum_db",
            "68225d213c44812fe844cd295d8821127c3cf31682be4eb19303cb53f9b147f7",
        ),
    ] {
        assert_eq!(hashes[name], expected);
    }
    let mut count = 0;
    for block in spec.split("```\n").skip(1) {
        if block.starts_with("isa=0.8\n") {
            let envelope = block.split("```").next().unwrap();
            assert!(
                envelopes.iter().any(|e| e == envelope),
                "printed envelope differs byte-for-byte"
            );
            count += 1;
        }
    }
    assert_eq!(count, 2);
}

#[test]
fn object_hash_excludes_only_the_three_nonsemantic_sections() {
    let raw = include_str!("fixtures/inc8.bisa");
    let base = Object::parse(raw).unwrap().identity().unwrap();
    let annotated = format!("{raw}\nsection coverage\n{{\"entries\":[]}}\n");
    let after = Object::parse(&annotated).unwrap().identity().unwrap();
    assert_eq!(base.object_hash, sha256_hex(raw.as_bytes()));
    assert_eq!(base.object_hash, after.object_hash);
    assert!(
        Object::parse(&raw.replace("lift.const.i8 1", "lift.const.i8 2"))
            .unwrap()
            .check_hashes()
            .is_err()
    );
}

#[test]
fn reference_order_and_self_scc_are_observable() {
    let raw = include_str!("fixtures/sum_db.bisa");
    let base = Object::parse(raw).unwrap().identity().unwrap();
    let env = &base.envelopes["sum"];
    assert!(env.contains("lift.call @$0"));
    assert_eq!(
        base.interface_hashes["sum"],
        sha256_hex(format!("{}:0", sha256_hex(env.as_bytes())).as_bytes())
    );
    let shifted = Object::parse(&raw.replace("\"base\":16,\"image\"", "\"base\":17,\"image\""))
        .unwrap()
        .identity()
        .unwrap();
    assert_ne!(
        base.interface_hashes["sum"],
        shifted.interface_hashes["sum"]
    );
    assert_ne!(
        base.interface_hashes["sum_db"],
        shifted.interface_hashes["sum_db"]
    );
    assert_eq!(
        base.interface_hashes["zero"],
        shifted.interface_hashes["zero"]
    );
}
