//! Semantic effective controls (§2): physical scalar wires with polarity.
//! Callers resolve names first. ABI operands and lexical push/pop stacks must
//! never pass through this set constructor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EffectiveControls(Vec<(String, bool)>);
impl EffectiveControls {
    pub(crate) fn new(literals: impl IntoIterator<Item = (String, bool)>) -> Self {
        let mut literals: Vec<_> = literals.into_iter().collect();
        literals.sort();
        literals.dedup();
        Self(literals)
    }
    pub(crate) fn literals(&self) -> &[(String, bool)] {
        &self.0
    }
    pub(crate) fn into_literals(self) -> Vec<(String, bool)> {
        self.0
    }
}
