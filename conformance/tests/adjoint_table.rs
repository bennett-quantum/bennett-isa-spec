use bennett_isa_conformance::{cases_dir, default_bin, load_cases, run_case, Outcome};

#[test]
fn adjoint_table_published_and_malformed_witness_cases() {
    let cases = load_cases(&cases_dir()).unwrap();
    let temporary = std::path::PathBuf::from(std::env::var("TMPDIR").unwrap())
        .join("adjoint-table-conformance");
    std::fs::create_dir_all(&temporary).unwrap();
    let mut seen = 0;
    for case in cases
        .iter()
        .filter(|c| c.id.starts_with("F-adjoint-table-"))
    {
        assert!(
            matches!(run_case(&default_bin(), case, &temporary), Outcome::Pass),
            "{}",
            case.id
        );
        seen += 1;
    }
    assert_eq!(seen, 2);
}
