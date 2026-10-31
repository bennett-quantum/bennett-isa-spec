use bennett_isa_ref::json::Json;
pub const Y: &str = include_str!("r14-witnesses/r14-Y-number.bisa");
pub const TABLE: &str = include_str!("r14-witnesses/r14-table-positive.bisa");
pub const AH: &str = "2fcd5b3d60b42ba6092447586a2157bf44ba3a6479ab2eb319bb67665feab8be";
pub const IH: &str = "8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93";
pub fn replace(base: &str, path: &[&str], next: Json) -> String {
    let (prefix, line) = base.split_once("section certificate\n").unwrap();
    let mut cert = Json::parse(line.trim_end()).unwrap();
    let mut node = &mut cert;
    for key in path {
        node = match node {
            Json::Object(m) => m.get_mut(*key).unwrap(),
            Json::Array(a) => &mut a[key.parse::<usize>().unwrap()],
            _ => panic!("container"),
        };
    }
    *node = next;
    format!("{prefix}section certificate\n{}\n", cert.canonical())
}
pub fn y_control() -> String {
    replace(
        Y,
        &["regions", IH, "claims", "8", "dual", "0", "Y"],
        Json::parse(r#"[["0","0","0","0"]]"#).unwrap(),
    )
}
pub fn negatives_y() -> Vec<String> {
    let good = y_control();
    (0..4)
        .flat_map(|i| {
            ["0", "false", "null", "[]", "{}"].map(|s| {
                replace(
                    &good,
                    &[
                        "regions",
                        IH,
                        "claims",
                        "8",
                        "dual",
                        "0",
                        "Y",
                        "0",
                        &i.to_string(),
                    ],
                    Json::parse(s).unwrap(),
                )
            })
        })
        .collect()
}
pub fn negatives_table() -> Vec<String> {
    [
        "[0]",
        r#"["wrong"]"#,
        "[false]",
        "[null]",
        "[{}]",
        "[[false]]",
        "[[null]]",
        "[[[]]]",
        "[[{}]]",
        "[[1.5]]",
        "[[-1]]",
        r#"[["wrong"]]"#,
        r#"[["1"]]"#,
    ]
    .map(|s| {
        replace(
            TABLE,
            &["regions", AH, "claims", "0", "expected", "table"],
            Json::parse(s).unwrap(),
        )
    })
    .to_vec()
}
