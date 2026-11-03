mod support;
use bennett_isa_ref::{Budget, Object, OracleInput, OracleMachine};

fn object(version: &str) -> Object {
    let text = support::object_with_data(&[("free",
        r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#,
        "oracle @free (x:wire in) (t:wire) (f:wire[4]) () ()\nregs { n:reg[8], p:reg[8], one:reg[1] }\n^entry():\nn = lift.const.i8 1\np = lift.alloc n\nlift.free p\nlift.boundscheck p 0\none = lift.const.i1 1\nlift.retval one")],&[("empty","")],
        "\nsection layout\n{\"A_tab\":2,\"H\":3,\"S_cells\":0,\"globals\":{\"empty\":{\"base\":4,\"image\":\"empty\",\"size\":0}},\"heap_base\":4,\"heap_limit_max\":1,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":8}\n")
        .replace("ptr_width=32","ptr_width=8").replace("0.8",version);
    support::rehash(&text)
}

#[test]
fn isa09_free_excludes_pinned_same_base_entries_and_preserves_08_results() {
    for version in ["0.8", "0.9"] {
        let object = object(version);
        object.check_hashes().unwrap();
        for steps in 0..=8 {
            let budget = Budget {
                steps,
                heap: 1,
                stack: 0,
                depth: 0,
            };
            // The zero-sized pinned global shares the allocated heap base.
            // After free it alone can witness boundscheck(p,0). The old08
            // instruction toggles it dead; E17.13's09 instruction excludes it.
            let expected = if version == "0.8" && steps >= 4 {
                (0, 7)
            } else if steps < 6 {
                (0, 1)
            } else {
                (1, 0)
            };
            for input in 0..2 {
                let input = OracleInput::bits(&[input]);
                let result = object.run_oracle("free", &input, budget).unwrap();
                assert_eq!(
                    (result.payload, result.code),
                    expected,
                    "{version}/B={steps}"
                );
                let mut machine = OracleMachine::new(&object, "free", &input, budget).unwrap();
                let entry = machine.snapshot();
                for _ in 0..steps {
                    machine.forward_step().unwrap();
                }
                for target in 0..2 {
                    for fault in 0..16 {
                        assert_eq!(
                            machine.commit(target, fault).unwrap(),
                            (target ^ expected.0, fault ^ expected.1)
                        );
                    }
                }
                for _ in 0..steps {
                    machine.inverse_step().unwrap();
                }
                assert_eq!(machine.snapshot(), entry, "{version}/B={steps}");
            }
        }
    }
}

#[test]
fn pinned_same_base_source_circuit_matches_both_engines_and_cleans_all_workspace() {
    use bennett_isa_conformance::{json, ref_catalog::Machine, ref_language::permutation};
    use bennett_isa_ref::ref_library::OracleInvocation;
    // Keep the native 8-bit case above unchanged. A separate compact address
    // space makes the complete source circuit small enough to check directly.
    let text = object("0.9")
        .text
        .replace("ptr_width=8", "ptr_width=2")
        .replace("\"ptr_width\":8", "\"ptr_width\":2")
        .replace("\"H\":3", "\"H\":2")
        .replace("\"base\":4", "\"base\":2")
        .replace("\"heap_base\":4", "\"heap_base\":2")
        .replace("reg[8]", "reg[2]")
        .replace("const.i8", "const.i2");
    let object = support::rehash(&text);
    let args = json::parse(r#"["x","t","f"]"#).unwrap();
    for steps in [4, 6] {
        let budget = Budget {
            steps,
            heap: 1,
            stack: 0,
            depth: 0,
        };
        let context = json::parse(&format!(r#"{{"token":["{steps}","1","0","0"]}}"#)).unwrap();
        let call = OracleInvocation::capture(
            &object,
            "free",
            Some(&context),
            budget,
            args.as_arr().unwrap(),
        )
        .unwrap();
        let bundles = [("x", 1), ("t", 1), ("f", 4)];
        let leaves = call
            .expand_source(&object.bound_ref_library().unwrap(), &bundles)
            .unwrap();
        let machine = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
        let mut state = machine.packed_state();
        for lane in 0..64 {
            machine.set_lane(&mut state, "x", (lane & 1) as u64, lane);
            machine.set_lane(&mut state, "t", ((lane >> 1) & 1) as u64, lane);
            machine.set_lane(&mut state, "f", (lane >> 2) as u64, lane);
        }
        let entry = state.clone();
        machine.run_packed(&mut state, u64::MAX, false).unwrap();
        for lane in 0..64 {
            let input = OracleInput::bits(&[(lane & 1) as u128]);
            let result = object.run_oracle("free", &input, budget).unwrap();
            assert_eq!(
                (result.payload, result.code),
                if steps == 6 { (1, 0) } else { (0, 1) }
            );
            assert_eq!(machine.get_lane(&state, "x", lane), (lane & 1) as u64);
            assert_eq!(
                machine.get_lane(&state, "t", lane),
                ((lane >> 1) & 1) as u64 ^ result.payload as u64
            );
            assert_eq!(
                machine.get_lane(&state, "f", lane),
                (lane >> 2) as u64 ^ result.code as u64
            );
        }
        machine.run_packed(&mut state, u64::MAX, true).unwrap();
        assert_eq!(state, entry, "B={steps}: full source inverse");
    }
}
