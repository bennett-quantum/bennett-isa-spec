//! K89 ordered union mechanics on already resolved product components. This
//! does not resolve producer occurrence bindings or establish normative D_F.
use super::{enumerate, Component, Domain, Xoshiro256};
use crate::{err, BitVector, Result};
use std::collections::BTreeSet;

pub struct OrderedEntryUnion {
    entries: Vec<Vec<BitVector>>,
}
impl OrderedEntryUnion {
    /// Products are in normative visit order; each component has inclusive
    /// bounds. The caller supplies resolved bundles/borrowed/regions/workspace
    /// in their §12.2 order. Last component varies fastest.
    pub fn from_products(products: &[Vec<(BitVector, BitVector)>]) -> Result<Self> {
        if products.is_empty() {
            return err("§12.2", "entry_products union must be nonempty");
        }
        let widths = products[0]
            .iter()
            .map(|(lo, _)| lo.width())
            .collect::<Vec<_>>();
        let mut entries = Vec::new();
        let mut seen = BTreeSet::new();
        for product in products {
            if product.len() != widths.len() {
                return err(
                    "§12.2",
                    "entry products resolve to different complete tuple shapes",
                );
            }
            let mut components = Vec::new();
            for (i, (lo, hi)) in product.iter().enumerate() {
                if lo.width() != widths[i]
                    || hi.width() != widths[i]
                    || lo.unsigned_cmp(hi)?.is_gt()
                {
                    return err("§12.2", "invalid entry product component width or range");
                }
                components.push((Component::Bundle(0, i), lo.width(), lo.clone(), hi.clone()));
            }
            for entry in enumerate(&Domain {
                borrowed_widths: Vec::new(),
                components,
                covers_d: false,
            })? {
                let key = entry.iter().map(BitVector::to_decimal).collect::<Vec<_>>();
                if seen.insert(key) {
                    crate::reserve_more(&mut entries, 1, "§12.2", "ordered union entries")?;
                    entries.push(entry);
                }
            }
        }
        Ok(Self { entries })
    }
    pub fn entries(&self) -> &[Vec<BitVector>] {
        &self.entries
    }
    /// One synthetic [0,N-1] rejection-sampler draw; no product choice or
    /// per-component words. N=1 consumes zero words in the shared sampler.
    pub fn draw_index(&self, rng: &mut Xoshiro256) -> Result<usize> {
        let hi = self.entries.len() - 1;
        let width = (usize::BITS - hi.leading_zeros()).max(1) as usize;
        let result = rng.draw(
            width,
            &BitVector::zero(width)?,
            &BitVector::from_u128(width, hi as u128)?,
        )?;
        usize::try_from(result.to_u128()?)
            .map_err(|_| crate::resource_error("§12.2", "union rank exceeds host range"))
    }
    /// All claimed-domain B values for the primary entry's fixed non-B tuple.
    /// This does not draw samples or count auxiliary replay as primary coverage.
    pub fn borrow_fiber(&self, index: usize, borrowed: &[usize]) -> Result<Vec<usize>> {
        let primary = self
            .entries
            .get(index)
            .ok_or_else(|| crate::Error::new("§12.2: primary rank outside union".into()))?;
        let positions = borrowed.iter().copied().collect::<BTreeSet<_>>();
        if positions.len() != borrowed.len() || positions.iter().any(|i| *i >= primary.len()) {
            return err("§12.2", "invalid borrowed component positions");
        }
        let mut fiber = Vec::new();
        for (i, row) in self.entries.iter().enumerate() {
            if row
                .iter()
                .zip(primary)
                .enumerate()
                .all(|(j, (a, b))| positions.contains(&j) || a == b)
            {
                crate::reserve_more(&mut fiber, 1, "§12.2", "borrow fiber ranks")?;
                fiber.push(i);
            }
        }
        Ok(fiber)
    }
}
