//! Council round 1: codex B18 (a fixture audit that audits every fixture), and
//! fable's suggestion that the §16.3 fixtures equal the spec text byte for byte.
mod support;
use bennett_isa_ref::Object;
use std::path::PathBuf;

fn fixtures() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut out: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bisa"))
        .collect();
    out.sort();
    out
}

#[test]
fn b18_every_bisa_file_is_audited_and_has_a_comments_file() {
    let all = fixtures();
    // Inventory: the audit covers whatever is committed, never a hand list.
    assert!(all.len() >= 12, "fixture inventory shrank: {all:?}");
    for path in &all {
        let text = std::fs::read_to_string(path).unwrap();
        Object::parse(&text)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
            .check_hashes()
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert!(
            path.with_extension("comments").exists(),
            "{} lacks its hand-derived .comments file",
            path.display()
        );
    }
}

#[test]
fn b18_a_corrupted_stored_hash_is_detected_by_the_same_audit() {
    let path = fixtures()
        .into_iter()
        .find(|p| p.ends_with("tiny_memory.bisa"))
        .unwrap();
    let text = std::fs::read_to_string(path).unwrap();
    let start = text.find("\"interface_hash\":\"").unwrap() + 18;
    let mut corrupted = text.clone();
    let flipped = if &text[start..start + 1] == "0" {
        "1"
    } else {
        "0"
    };
    corrupted.replace_range(start..start + 1, flipped);
    let o = Object::parse(&corrupted).unwrap();
    assert!(o.check_hashes().unwrap_err().to_string().contains("§9.2"));
}

/// The n-th fenced block after `marker` in the specification.
fn spec_block(marker: &str, n: usize) -> String {
    let spec = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../spec/Bennett-ISA-Spec.md"),
    )
    .unwrap();
    let mut rest = &spec[spec.find(marker).unwrap()..];
    let mut block = String::new();
    for _ in 0..=n {
        let open = rest.find("```\n").unwrap() + 4;
        let close = rest[open..].find("```").unwrap() + open;
        block = rest[open..close].to_string();
        rest = &rest[close + 3..];
    }
    block
}

#[test]
fn published_example_objects_are_the_fixtures_byte_for_byte() {
    assert_eq!(
        spec_block("**Example 1** (`inc8`)", 0),
        include_str!("fixtures/inc8.bisa")
    );
    assert_eq!(
        spec_block("**Example 2** (`sum_db`)", 0),
        include_str!("fixtures/sum_db.bisa")
    );
}

#[test]
fn fixture_identities_are_pinned_as_literals() {
    // muse suggestion: the stored hashes are re-derived by the crate itself
    // (support::rehash); pinning them here makes any future envelope change a
    // visible test failure rather than a silent re-derivation.
    let pinned: &[(&str, &str, &str)] = &[
        (
            "actions",
            "bell",
            "3b0881575762fa4e1fd5b2a160cd6abfcbfe60f011df3b1968df7e44de11e1d3",
        ),
        (
            "actions",
            "flip",
            "7b3212f6647103050fa24c34e767afd19fbe3009134d6f5c7280a400ff9ef398",
        ),
        (
            "actions",
            "image",
            "90dbeda2a04d9f5ac10570fa079aaf22540fb1bf8da780415dcd238334996aa4",
        ),
        (
            "actions",
            "lifted",
            "2acd73fbb764d1a278bcba54e7a13c99ddd1d159e8f63ea59f3af7338e313f67",
        ),
        (
            "actions",
            "loop",
            "56ab73b8a0938e882c2a3293d4fb401a07bbbb931485b88948973ba1c3dc1597",
        ),
        (
            "actions",
            "memory",
            "62c70737b8fcae09329da78eed2ed48aad61dfe0137e2d8d02c1e1ffb9dc5f76",
        ),
        (
            "arithmetic",
            "add",
            "7c80a06412db38f658cf0441a0f64529cbfb8b903814f239cad3ba770f99308a",
        ),
        (
            "arithmetic",
            "and",
            "bf8e3bb2f7c71c884f5632c3ae27be18ba6854bac45ec83d3d7b5ab42d9160ac",
        ),
        (
            "arithmetic",
            "ashr",
            "2dc127ee704b144c5ff8110fbce24ebfc8ae148670deb64dbb7c458de84ff5ca",
        ),
        (
            "arithmetic",
            "badjump",
            "0b0f368147c4bf9c5fcffc84abb37ee1775504d13204ed6fcd6b19b735c00abf",
        ),
        (
            "arithmetic",
            "casts",
            "df2e1f2c1b24610d7e868f6f8a4cb1865e7c05f9637ae1e0438f93636eed4f11",
        ),
        (
            "arithmetic",
            "eq",
            "2f2605b2208e04ed6f91cf3f8375b9c70839038c0c8209cac79bee33280d152c",
        ),
        (
            "arithmetic",
            "fault0",
            "1f5780fc3ed3345fb966ec20479590f11403222b2889dc8a76ad6e2cb9026f25",
        ),
        (
            "arithmetic",
            "fault1",
            "569088300ff8a05a82e69a1fa7af2569c976892b46f33cf4256debbe49f28c88",
        ),
        (
            "arithmetic",
            "fault10",
            "9331bcea7de7b1492be6918c6e29e5635a5f9faf6d54cce3a498cae23b7e83e3",
        ),
        (
            "arithmetic",
            "fault2",
            "8012e4889e407a4fd174d943b4ca9cd3281a008c50ceaccb24a8566529fd2a18",
        ),
        (
            "arithmetic",
            "fault3",
            "64962111607ff89d5875a2044ee582d558ff1e51570a1703d8541eb15b0aef67",
        ),
        (
            "arithmetic",
            "fault4",
            "1025848f6bd2dcf9697b2bf1933aa9b4ce8293ec7c252538c92d5d04a295d2e8",
        ),
        (
            "arithmetic",
            "fault5",
            "7e21c6f8f21d6746da6357344ca5aaf21e84d50f6352970744eb052a21158c16",
        ),
        (
            "arithmetic",
            "fault6",
            "e3174365e07ee50aea728a15c1529eb69d3dd71ee0189e86135ce59336b99654",
        ),
        (
            "arithmetic",
            "fault7",
            "feee076120be5fb4b5c60870794a5951fc02bd764996b00b5a98fcf4df7c21b7",
        ),
        (
            "arithmetic",
            "fault8",
            "3f9151ece9237c1ffc912095056f4e03331d1b4e05d77b0f64985f95d07248ee",
        ),
        (
            "arithmetic",
            "fault9",
            "a953e00368a00f72dd71de2f704258d4c4ede6e46a11c43400f117277befbd3d",
        ),
        (
            "arithmetic",
            "jump",
            "0dc6f86ee00be08f2a9a93b13c3bb15f10fd6c5fb7724afc50d818525b2ca908",
        ),
        (
            "arithmetic",
            "lshr",
            "c5e57c21e06831cfcfb142078cb72ef2366f1c5bc591808001eb7ea3601594c4",
        ),
        (
            "arithmetic",
            "mul",
            "3201940a9efdc65fe248f48883412904740b49486dba3bce20b2c75230ce80f7",
        ),
        (
            "arithmetic",
            "ne",
            "8ec6f6ce7f64dc1bfb699e6166c919b59de152d8b7227783c89eb1f87b87886c",
        ),
        (
            "arithmetic",
            "or",
            "fbfd5ca4f0d388fd3e9c51f7be47eac64bbd208760ee23b77778c2d793c28391",
        ),
        (
            "arithmetic",
            "parmov",
            "dfa3e93a694fd26506ab85fdb5c91cc2f7240e5c67ac2f96489aff4415ae43cd",
        ),
        (
            "arithmetic",
            "sdiv",
            "115fb415f346d9066d5edd800ec3710fea7d6e5da3baf10491341d097f900fae",
        ),
        (
            "arithmetic",
            "select",
            "8666f45532a3e411d9b74245fbac9d868d9bb74b58c35cf48dc507f38c0fb8df",
        ),
        (
            "arithmetic",
            "shl",
            "f58b391c05e70784bbb6da613f5a690282026fca38f7a3409bb07e1bb527bc5a",
        ),
        (
            "arithmetic",
            "sle",
            "fad9f7b527b8a6142f938078be56cedf05333583633636031027cd0286352e42",
        ),
        (
            "arithmetic",
            "slt",
            "c77dfe5f449851d9c44b4f06070166c6f1161f5ff42e6ebe9824776dca18ee31",
        ),
        (
            "arithmetic",
            "srem",
            "02a4afa08d8a425d1d7a3737c8da3a2c27b5d9fcaf528ba17f46e4bcaa4b87e2",
        ),
        (
            "arithmetic",
            "sub",
            "080d1d22642a043ef267f4f6814af1591511d8f746ecb494b087878ce8397e7c",
        ),
        (
            "arithmetic",
            "udiv",
            "394b35c6548da71eb06f4ebbdf620c7fe900eff4359464c51216f3dc07a9cc62",
        ),
        (
            "arithmetic",
            "ule",
            "89739797986f1edc445035af8de35eedfe4495fe9d1e7e91ae626ae9004c2f11",
        ),
        (
            "arithmetic",
            "ult",
            "84f277fc86190b7d3986429b5f8958057c574dccbb5a652ca9a6de7e827702b4",
        ),
        (
            "arithmetic",
            "urem",
            "47ca0d92dc35e402faca3b1bce44bc6b9d2f047bdf5921cd40093eeca9aa3f45",
        ),
        (
            "arithmetic",
            "xor",
            "c2d37f0c1c6740bb8ca00e7a0cd766101a34953e97353e9bc883b27a654e8adf",
        ),
        (
            "classical",
            "divzero",
            "30fd03864fe91c61b0c8aa2ce72a158e1647c02a9dc655fa1018c5196b4efbfd",
        ),
        (
            "classical",
            "effects",
            "f1344e767c6ce40993c3c6164eba4d58ac6bb8bbbfdba33de7400aa795d4c0b9",
        ),
        (
            "classical",
            "entropy_short",
            "260aa61d5d96b991e946e2fd4986fe9e319e2c43ef374e8f90d85c112b3d5b88",
        ),
        (
            "classical",
            "f",
            "564cbe6a49414643ca03211d12c78f3cf431ba3d2ecf0c8b06ea1aafc0392389",
        ),
        (
            "classical",
            "g",
            "e8a3dc5429049b16559aa6a4262779e3dcd37a758967def4e08c371a5b765fff",
        ),
        (
            "classical",
            "host",
            "6ef7b54c8ab5641636211b2d43f0df86cfbc129916db3b25b805e7ad33b97774",
        ),
        (
            "classical",
            "host_invalid",
            "cd54f756eb0a65f3c2881453c9c7e03d44e1ae9c651ca01efdeede661b3fb573",
        ),
        (
            "classical",
            "impure",
            "c8b90ca42cbf14f0ac90c4395b94100ddbe1319688931014e900acb2a4694840",
        ),
        (
            "classical",
            "memfault",
            "818b6f38c57d9afe5fbad77a00457a7f9f3484af7f3d25e3b6238b6ea301ba7e",
        ),
        (
            "classical",
            "nested",
            "e0db43691a75656b7a37a8156ca44cfa17523a63c0e2a36e20433869c7b1112b",
        ),
        (
            "classical",
            "sum",
            "0fe8c7991be189017d08024a7b880f04646a46bd0831a4de093cd99a063918ab",
        ),
        (
            "classical",
            "unknown_effect",
            "9ccebed66dd2a159125d50818ef06c82b8512612940a695605826fd3452d3524",
        ),
        (
            "control_flow",
            "borrowed",
            "4e3eba45f290de3007ae1ee11cd6ceb1e90cd9d64391e163255edd90eee78ef8",
        ),
        (
            "control_flow",
            "bytes",
            "f7f06a09d59c31dcbff6f80485ab1c7fc3974bbdfb9e1e6b813bede211a94a6f",
        ),
        (
            "control_flow",
            "conditional",
            "9a71ebdeed0fcf79499b855cf2d7dfc747680ba58c06ebe130eb9576562205c9",
        ),
        (
            "control_flow",
            "controlled_phase",
            "ab91d03274c13ebfeb0832746e0567b6fe2e3178d34300ffb353cfc1cb2fbc06",
        ),
        (
            "control_flow",
            "exchange",
            "bf13912ffa2d6932f730622105f5a243eec9a3c01554a034e0e97d0ca7a195f2",
        ),
        (
            "control_flow",
            "image",
            "fd66ff5ee2580e4f999e4e85e74e2e24de2b54ffa1c70c4a6b90dc63e56a8e90",
        ),
        (
            "control_flow",
            "phase",
            "eb4b8c89a878e6fb8c514214417d0827f809fe0fd6b155e2f395ec19f5f31271",
        ),
        (
            "control_flow",
            "while3",
            "70c70c733a7c87eeedb2b9d3a77fea16cb1240041dddcfb865e8149b50769883",
        ),
        (
            "heap",
            "afterfree",
            "b4e19ac0b85b4c81d01de0e0e8f15b5a631fdaec8dca6680c17175e81ea945d7",
        ),
        (
            "heap",
            "badstore",
            "b4147fd646bdfdc270596c04cb445561fe401774e1fdd178fcde3e77a5699725",
        ),
        (
            "heap",
            "doublefree",
            "86f87b18056089dd5ab1648a9843bb8f69d3cb6ac70755866be63ef068c425d4",
        ),
        (
            "heap",
            "faultif",
            "3ef6c026118750eebb4c309eadf18c20eab4daab5aa5ef05b25dd7861238cd50",
        ),
        (
            "heap",
            "g",
            "f5c5556f79f032f7a0607387d69cab7bbb21988ca15461645340abb11e562813",
        ),
        (
            "heap",
            "heap",
            "30660357457354b768b86e153f366ead95aeac5d125e860d8e6cb8743f8d35a8",
        ),
        (
            "heap",
            "outoftable",
            "a06203de444a4a9a9a34a6781efee8e6290b27d79ed102165e5e2edd571fd56f",
        ),
        (
            "heap",
            "pinned",
            "7fbc54ee4a45cf4ebaeb84b5d9344b3fa04906b125784404342055b075509197",
        ),
        (
            "heap",
            "zeroalloc",
            "415e0ce86806f048d197ed864b321091a00cc27575e56b7661a85958ad17b20d",
        ),
        (
            "inc8",
            "inc8",
            "8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93",
        ),
        (
            "mutual",
            "f",
            "e4028cf7347c2779e19c8193ef1039837944acebd4784e9c3a42fbe236e737ec",
        ),
        (
            "mutual",
            "g",
            "7dc9c25aff5230a46ae697edf187d34ba72a03f7a3c1a520c6fc6bf5462c6374",
        ),
        (
            "mutual",
            "main",
            "598fdf171d0a201d52e8fbe2339c70d1a26b36a72dcd60503c87c51dff88039b",
        ),
        (
            "nested",
            "cancel",
            "c98f82615ce05c7f82ee89d9106ae0655f3396332ad2a6616f63266c03043c34",
        ),
        (
            "nested",
            "ignore",
            "9687f21af62158ff87ab8a974e056774ee75b531e158cc4538a11d4631f89012",
        ),
        (
            "nested",
            "inc8",
            "8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93",
        ),
        (
            "nested",
            "nested",
            "562ad89a1befd9629409f21362684d92dc9d57b93968b1d6d5a868e6e2bc6afd",
        ),
        (
            "registers",
            "allocation",
            "20449c3d1c45ec590a7f7b356ed59719e2daaec9bbaa18f8da2eaa43dfab0008",
        ),
        (
            "registers",
            "bad_uncall",
            "1a392c2bfb917334f6ed3266c24b5d1a638d08371985d3d4c4def835cad24d74",
        ),
        (
            "registers",
            "bell",
            "0f633b7a5f72475dc249883c0b8c6a3c91bfe62d40de642013afb5bd2015f149",
        ),
        (
            "registers",
            "measurement",
            "dfc970cdea49da3834707362043d64ff084bcb641da635034e4cce0271b2943f",
        ),
        (
            "registers",
            "measuring",
            "f587dd8342b03b99c80772b046c324dfe63872a8c58604264d73605dbecf505b",
        ),
        (
            "registers",
            "toggle",
            "de3621c3dee0d970a86479e38ff4088730604a5b4d16429b71af2ec85ab0f42f",
        ),
        (
            "registers",
            "unclean",
            "fe0d76be58a95d67e2619cd08f33cd9e25ba530a09d6fa848d6bf955e162f56b",
        ),
        (
            "registers",
            "views",
            "6ea78656c92cd787bea86133572e2c9766d3f70208676cad0c043731c0fd6102",
        ),
        (
            "sum_db",
            "sum",
            "74f110bbafde68a4785025e7e571ef871ddd83a09465ab9c1604f653a0cfcd92",
        ),
        (
            "sum_db",
            "sum_db",
            "68225d213c44812fe844cd295d8821127c3cf31682be4eb19303cb53f9b147f7",
        ),
        (
            "sum_db",
            "zero",
            "7002ed5ea010907a0ae99065663f22fb7d18ca1d6cb171fed57e2594bd9367e6",
        ),
        (
            "tiny_memory",
            "tiny",
            "be21636357102b2a324f99db1bd058ad82f1c2eb3337d99f22a2533a32436142",
        ),
        (
            "wide",
            "add257",
            "19caea9bf13ee820b20984eb202af22e98854338922cb4ee16067f8a1ea6e3d9",
        ),
        (
            "wide",
            "call257",
            "894aacbd0118f47d7f7ed3775d41d9195ac9b1ea06145285a60b8de41df39f89",
        ),
        (
            "wide",
            "div257",
            "612b674cafa8e7a4035dbb8fcd7fa9ff214438e85280d8fdfdb7de8bb6e98f00",
        ),
        (
            "wide",
            "fault257",
            "a6ea0a70d8b01bb10a013ea440b7cd323dab09f30b1c63fcee4419c96230c8fe",
        ),
        (
            "wide",
            "fault_small257",
            "ec67bbabf748db355d4bd498848fc8743a7b01516c6fa0acbf3c69a24e3827f1",
        ),
        (
            "wide",
            "id257",
            "0755279a5f665b6bb03930ed4b5a7fb376aafa5bc3cebb737a1b20d3d8139332",
        ),
        (
            "r9-sol-oracle",
            "o",
            "a8f4a15663aead9ecb6618fe3847deba546cf3172f645ffc7e763f551e529e27",
        ),
        (
            "r9-sol-oracle",
            "a",
            "4cabd490b0d1a8dfe31489ec234e91867e66b54c6c4d030eb4e8d1cf6f84bb38",
        ),
        (
            "r9-sol-qinplace",
            "o",
            "a8f4a15663aead9ecb6618fe3847deba546cf3172f645ffc7e763f551e529e27",
        ),
        (
            "r9-sol-qinplace",
            "a",
            "bc6e5d89276a62eec7074a716d8dbce40e94a2c80c287ad8fc5b5e5c6c756fbf",
        ),
        (
            "r9-codex-oracle",
            "o",
            "cab76c914fa09a3fc8237381c6de91e83eeaefbd65cb7f52571ed4020abe8926",
        ),
        (
            "r9-codex-oracle",
            "a",
            "be1f0e087e06fecd99df7c3cbb5c3c8c186fde8f814be1f826487e858fa66296",
        ),
        (
            "r9-loop",
            "a",
            "bb983f1dc5a8eea46912ea64d1226a9345a05d6cba6cc8452efc9f39b4a17a89",
        ),
    ];
    let mut checked = 0;
    for (fixture, symbol, hash) in pinned {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/{fixture}.bisa"));
        let o = Object::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
        assert_eq!(
            o.identity().unwrap().interface_hashes[*symbol],
            *hash,
            "{fixture}/{symbol}"
        );
        checked += 1;
    }
    let total: usize = fixtures()
        .iter()
        .map(|p| {
            Object::parse(&std::fs::read_to_string(p).unwrap())
                .unwrap()
                .symbols()
                .unwrap()
                .len()
        })
        .sum();
    assert_eq!(checked, total, "every fixture symbol is pinned");
}

#[test]
fn r10_independent_effective_control_and_grammar_pins() {
    for tag in ["controls", "duplicate", "cc", "r", "omitted-bounds"] {
        support::r10_pin(tag);
    }
    // Rejected objects still retain their literal byte digests. They must not
    // be silently rewritten into legal objects before hashing.
    for tag in [
        "bare-cc",
        "bare-r",
        "empty-bounds",
        "fn-oracle",
        "inc-empty-bounds",
        "empty-lhs",
    ] {
        let c = support::r10_case(tag);
        assert_eq!(
            bennett_isa_ref::sha256_hex(support::r10_text(tag).as_bytes()),
            c.get("object_hash").unwrap().string().unwrap()
        );
    }
}
