//! Canonical reduced ordered BDDs: a full-domain Boolean equality witness.
//! Terminals 0/1 and unique (variable,low,high) nodes make equal functions
//! identical. No sampling and no reference-interpreter dependency.
use std::collections::BTreeMap;
pub struct Bdd {
    nodes: Vec<(usize, usize, usize)>,
    unique: BTreeMap<(usize, usize, usize), usize>,
    cache: BTreeMap<(u8, usize, usize), usize>,
    variables: usize,
}
impl Bdd {
    pub fn new() -> Self {
        Self {
            nodes: vec![(usize::MAX, 0, 0), (usize::MAX, 1, 1)],
            unique: BTreeMap::new(),
            cache: BTreeMap::new(),
            variables: 0,
        }
    }
    fn node(&mut self, v: usize, l: usize, h: usize) -> usize {
        if l == h {
            return l;
        }
        let key = (v, l, h);
        if let Some(n) = self.unique.get(&key) {
            return *n;
        }
        let n = self.nodes.len();
        self.nodes.push(key);
        self.unique.insert(key, n);
        n
    }
    /// Exact number of satisfying assignments over all declared variables.
    /// Counts fail explicitly if the witness domain exceeds u128.
    pub fn satisfying_count(&self, root: usize) -> Result<u128, String> {
        fn count(
            b: &Bdd,
            n: usize,
            level: usize,
            memo: &mut BTreeMap<(usize, usize), u128>,
        ) -> Result<u128, String> {
            if n == 0 {
                return Ok(0);
            }
            if n == 1 {
                return 1u128
                    .checked_shl(
                        (b.variables - level)
                            .try_into()
                            .map_err(|_| "count width")?,
                    )
                    .ok_or("BDD count overflow".into());
            }
            if let Some(v) = memo.get(&(n, level)) {
                return Ok(*v);
            }
            let (variable, low, high) = b.nodes[n];
            let sum = count(b, low, variable + 1, memo)?
                .checked_add(count(b, high, variable + 1, memo)?)
                .ok_or("BDD count overflow")?;
            let skipped = variable - level;
            let factor = 1u128
                .checked_shl(skipped.try_into().map_err(|_| "count width")?)
                .ok_or("BDD count overflow")?;
            let result = sum.checked_mul(factor).ok_or("BDD count overflow")?;
            memo.insert((n, level), result);
            Ok(result)
        }
        if root >= self.nodes.len() {
            return Err("invalid BDD root".into());
        }
        count(self, root, 0, &mut BTreeMap::new())
    }
    pub fn variable(&mut self) -> usize {
        let v = self.variables;
        self.variables += 1;
        self.node(v, 0, 1)
    }
    pub fn xor(&mut self, a: usize, b: usize) -> usize {
        self.apply(0, a, b)
    }
    pub fn and(&mut self, a: usize, b: usize) -> usize {
        self.apply(1, a, b)
    }
    pub fn or(&mut self, a: usize, b: usize) -> usize {
        self.apply(2, a, b)
    }
    pub fn not(&mut self, a: usize) -> usize {
        self.xor(a, 1)
    }
    fn apply(&mut self, op: u8, a: usize, b: usize) -> usize {
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        if a < 2 && b < 2 {
            return match op {
                0 => a ^ b,
                1 => a & b,
                2 => a | b,
                _ => unreachable!(),
            };
        }
        if a == b {
            return if op == 0 { 0 } else { a };
        }
        if a == 0 {
            return if op == 1 { 0 } else { b };
        }
        if a == 1 && op == 2 {
            return 1;
        }
        let key = (op, a, b);
        if let Some(n) = self.cache.get(&key) {
            return *n;
        }
        let (av, al, ah) = self.nodes[a];
        let (bv, bl, bh) = self.nodes[b];
        let v = av.min(bv);
        let low = self.apply(
            op,
            if av == v { al } else { a },
            if bv == v { bl } else { b },
        );
        let high = self.apply(
            op,
            if av == v { ah } else { a },
            if bv == v { bh } else { b },
        );
        let n = self.node(v, low, high);
        self.cache.insert(key, n);
        n
    }
    pub fn equal(&mut self, a: &[usize], b: &[usize]) -> usize {
        assert_eq!(a.len(), b.len());
        let mut v = 1;
        for (a, b) in a.iter().zip(b) {
            let diff = self.xor(*a, *b);
            let same = self.not(diff);
            v = self.and(v, same);
        }
        v
    }
    /// Mathematical unsigned order, tested from the most significant bit.
    pub fn less(&mut self, a: &[usize], b: &[usize]) -> usize {
        assert_eq!(a.len(), b.len());
        let mut lt = 0;
        let mut eq = 1;
        for (a, b) in a.iter().zip(b).rev() {
            let na = self.not(*a);
            let less = self.and(na, *b);
            let first = self.and(eq, less);
            lt = self.or(lt, first);
            let diff = self.xor(*a, *b);
            let same = self.not(diff);
            eq = self.and(eq, same);
        }
        lt
    }
    pub fn constant(value: usize, width: usize) -> Vec<usize> {
        (0..width).map(|i| (value >> i) & 1).collect()
    }
    pub fn add(&mut self, a: &[usize], b: &[usize]) -> Vec<usize> {
        assert_eq!(a.len(), b.len());
        let mut carry = 0;
        let mut out = Vec::new();
        for (a, b) in a.iter().zip(b) {
            let p = self.xor(*a, *b);
            out.push(self.xor(p, carry));
            let ab = self.and(*a, *b);
            let ac = self.and(*a, carry);
            let bc = self.and(*b, carry);
            let t = self.or(ab, ac);
            carry = self.or(t, bc);
        }
        out
    }
    pub fn equivalent(&mut self, a: usize, b: usize, domain: usize) -> bool {
        let diff = self.xor(a, b);
        self.and(diff, domain) == 0
    }
    pub fn evaluate(&self, mut node: usize, assignment: usize) -> bool {
        while node > 1 {
            let (v, l, h) = self.nodes[node];
            node = if assignment >> v & 1 == 0 { l } else { h };
        }
        node == 1
    }
    pub fn nodes(&self) -> usize {
        self.nodes.len()
    }
}

impl Default for Bdd {
    fn default() -> Self {
        Self::new()
    }
}
