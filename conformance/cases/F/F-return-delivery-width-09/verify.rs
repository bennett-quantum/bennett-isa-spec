use bennett_isa_conformance::{
    json,
    ref_language::{Expander, Registry},
};

#[test]
fn publication_fixes_complete_history_width() {
    let draft = include_str!("../../../../spec/Bennett-ISA-Spec-0.9.md");
    let marker = "**LIFT-RETURN-WIDTH witness**\n\n```json\n";
    let raw = draft
        .split_once(marker)
        .expect("missing return-width publication")
        .1
        .split_once("\n```")
        .unwrap()
        .0;
    assert_eq!(
        json::parse(raw).unwrap(),
        json::parse(include_str!("widths.json")).unwrap()
    );
    let listed = [4usize, 4, 2];
    let slots = [listed[0].max(8), listed[1], listed[2]];
    assert_eq!(slots, [8, 4, 2]);
    assert_eq!(2 * 3 + slots.iter().sum::<usize>() + 1, 21);
    assert_eq!(4 * 21, 84);
}

#[test]
fn unchanged_return_recipe_scans_the_full_declaration_width() {
    for n in [4, 8] {
        let args = json::parse(&format!(
            r#"[2,4,8,8,3,2,4,{n},1,
            "depth","fp","stack","dfp","ddepth","flag","faulted","code","v","delivery","succ",
            [{{"id":2,"width":1,"slot":0,"location":"v"}}],
            [{{"id":0,"width":1,"slot":0,"location":"root_r"}},
             {{"id":1,"width":8,"slot":1,"location":"ghost"}},
             {{"id":2,"width":1,"slot":0,"location":"v"}}]]"#
        ))
        .unwrap();
        let mut expander = Expander::new(
            include_str!("../../../../spec/ref-0.9.json"),
            Registry::new(),
        )
        .unwrap();
        let result = expander.expand(
            "lift.ret",
            args.as_arr().unwrap(),
            &[
                ("depth", 2),
                ("fp", 4),
                ("stack", 64),
                ("dfp", 4),
                ("ddepth", 2),
                ("flag", 1),
                ("faulted", 1),
                ("code", 4),
                ("v", 1),
                ("delivery", n),
                ("succ", 3),
                ("root_r", 1),
                ("ghost", 8),
            ],
        );
        if n == 4 {
            assert!(result.unwrap_err().contains("wire index out of range"));
        } else {
            assert!(result.is_ok(), "{:?}", result.err());
        }
    }
}
