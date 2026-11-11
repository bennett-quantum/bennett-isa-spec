use bennett_isa_ref::target::ModelOperands;

pub fn operands(own: usize, total: usize, targets: usize) -> ModelOperands {
    let own_controls = (0..own)
        .map(|bit| ("a-own".into(), bit))
        .collect::<Vec<_>>();
    let effective_controls = (0..total - own)
        .map(|bit| ("b-inherited".into(), bit))
        .collect::<Vec<_>>();
    let targets = (0..targets)
        .map(|bit| ("target".into(), bit))
        .collect::<Vec<_>>();
    let allocation_order = effective_controls
        .iter()
        .chain(&own_controls)
        .chain(&targets)
        .cloned()
        .collect();
    ModelOperands {
        own_controls,
        effective_controls,
        targets,
        allocation_order,
    }
}
