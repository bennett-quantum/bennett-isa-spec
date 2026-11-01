//! Persistent dynamic-instance environment. Checkpoints share one AVL root;
//! updates copy only a search path and share immutable instance-stack tails.
use crate::classical::ClassicalValue;
use std::{cmp::Ordering, sync::Arc};

type Key = (u64, String);
#[derive(Clone, Default)]
pub(crate) struct Stack(Option<Arc<Value>>);
struct Value {
    value: ClassicalValue,
    tail: Stack,
}
impl Stack {
    fn push(&self, value: ClassicalValue) -> Self {
        Self(Some(Arc::new(Value {
            value,
            tail: self.clone(),
        })))
    }
    pub(crate) fn last(&self) -> Option<&ClassicalValue> {
        self.0.as_ref().map(|v| &v.value)
    }
}
impl Drop for Stack {
    fn drop(&mut self) {
        // Long loop histories must not recursively drop one Rust frame/value.
        let mut next = self.0.take();
        while let Some(node) = next {
            match Arc::try_unwrap(node) {
                Ok(mut value) => next = value.tail.0.take(),
                Err(_) => break,
            }
        }
    }
}
impl PartialEq for Stack {
    fn eq(&self, other: &Self) -> bool {
        let (mut a, mut b) = (&self.0, &other.0);
        loop {
            match (a, b) {
                (Some(x), Some(y)) => {
                    if Arc::ptr_eq(x, y) {
                        return true;
                    }
                    if x.value != y.value {
                        return false;
                    }
                    (a, b) = (&x.tail.0, &y.tail.0);
                }
                (None, None) => return true,
                _ => return false,
            }
        }
    }
}
impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        let mut next = self.0.as_ref();
        while let Some(value) = next {
            list.entry(&value.value);
            next = value.tail.0.as_ref();
        }
        list.finish()
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Instances(Option<Arc<Node>>);
#[derive(Debug)]
struct Node {
    key: Key,
    values: Stack,
    left: Instances,
    right: Instances,
    height: u32,
}
impl Instances {
    fn height(&self) -> u32 {
        self.0.as_ref().map_or(0, |n| n.height)
    }
    fn node(key: Key, values: Stack, left: Self, right: Self) -> Self {
        let height = 1 + left.height().max(right.height());
        Self(Some(Arc::new(Node {
            key,
            values,
            left,
            right,
            height,
        })))
    }
    fn balanced(key: Key, values: Stack, mut left: Self, mut right: Self) -> Self {
        if left.height() > right.height() + 1 {
            let l = left.0.as_ref().unwrap();
            if l.right.height() > l.left.height() {
                let r = l.right.0.as_ref().unwrap();
                left = Self::node(
                    r.key.clone(),
                    r.values.clone(),
                    Self::node(
                        l.key.clone(),
                        l.values.clone(),
                        l.left.clone(),
                        r.left.clone(),
                    ),
                    r.right.clone(),
                );
            }
            let l = left.0.as_ref().unwrap();
            return Self::node(
                l.key.clone(),
                l.values.clone(),
                l.left.clone(),
                Self::node(key, values, l.right.clone(), right),
            );
        }
        if right.height() > left.height() + 1 {
            let r = right.0.as_ref().unwrap();
            if r.left.height() > r.right.height() {
                let l = r.left.0.as_ref().unwrap();
                right = Self::node(
                    l.key.clone(),
                    l.values.clone(),
                    l.left.clone(),
                    Self::node(
                        r.key.clone(),
                        r.values.clone(),
                        l.right.clone(),
                        r.right.clone(),
                    ),
                );
            }
            let r = right.0.as_ref().unwrap();
            return Self::node(
                r.key.clone(),
                r.values.clone(),
                Self::node(key, values, left, r.left.clone()),
                r.right.clone(),
            );
        }
        Self::node(key, values, left, right)
    }
    pub(crate) fn get(&self, key: &Key) -> Option<&Stack> {
        let mut next = self.0.as_ref();
        while let Some(node) = next {
            match key.cmp(&node.key) {
                Ordering::Less => next = node.left.0.as_ref(),
                Ordering::Greater => next = node.right.0.as_ref(),
                Ordering::Equal => return Some(&node.values),
            }
        }
        None
    }
    fn set(&self, key: Key, values: Stack) -> Self {
        let Some(node) = &self.0 else {
            return Self::node(key, values, Self::default(), Self::default());
        };
        match key.cmp(&node.key) {
            Ordering::Less => Self::balanced(
                node.key.clone(),
                node.values.clone(),
                node.left.set(key, values),
                node.right.clone(),
            ),
            Ordering::Greater => Self::balanced(
                node.key.clone(),
                node.values.clone(),
                node.left.clone(),
                node.right.set(key, values),
            ),
            Ordering::Equal => Self::node(key, values, node.left.clone(), node.right.clone()),
        }
    }
    fn remove(&self, key: &Key) -> Self {
        let Some(node) = &self.0 else {
            return self.clone();
        };
        match key.cmp(&node.key) {
            Ordering::Less => Self::balanced(
                node.key.clone(),
                node.values.clone(),
                node.left.remove(key),
                node.right.clone(),
            ),
            Ordering::Greater => Self::balanced(
                node.key.clone(),
                node.values.clone(),
                node.left.clone(),
                node.right.remove(key),
            ),
            Ordering::Equal => {
                if node.left.0.is_none() {
                    return node.right.clone();
                }
                if node.right.0.is_none() {
                    return node.left.clone();
                }
                let mut successor = node.right.0.as_ref().unwrap();
                while let Some(left) = &successor.left.0 {
                    successor = left;
                }
                Self::balanced(
                    successor.key.clone(),
                    successor.values.clone(),
                    node.left.clone(),
                    node.right.remove(&successor.key),
                )
            }
        }
    }
    pub(crate) fn push(&mut self, key: Key, value: ClassicalValue) {
        let values = self.get(&key).cloned().unwrap_or_default().push(value);
        *self = self.set(key, values);
    }
    pub(crate) fn pop(&mut self, key: &Key) -> bool {
        let Some(values) = self.get(key) else {
            return false;
        };
        let tail = values
            .0
            .as_ref()
            .expect("nonempty instance stack")
            .tail
            .clone();
        *self = if tail.0.is_none() {
            self.remove(key)
        } else {
            self.set(key.clone(), tail)
        };
        true
    }
    fn iter(&self) -> impl Iterator<Item = (&Key, &Stack)> {
        let mut path = Vec::new();
        let mut next = self.0.as_ref();
        std::iter::from_fn(move || {
            while let Some(node) = next {
                path.push(node);
                next = node.left.0.as_ref();
            }
            let node = path.pop()?;
            next = node.right.0.as_ref();
            Some((&node.key, &node.values))
        })
    }
    pub(crate) fn keys(&self) -> impl Iterator<Item = &Key> {
        self.iter().map(|(k, _)| k)
    }
}
impl PartialEq for Instances {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(a), Some(b)) if Arc::ptr_eq(a, b) => true,
            _ => self.iter().eq(other.iter()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    fn check(tree: &Instances) -> u32 {
        let Some(node) = &tree.0 else {
            return 0;
        };
        let (l, r) = (check(&node.left), check(&node.right));
        assert!(l.abs_diff(r) <= 1);
        assert_eq!(node.height, 1 + l.max(r));
        node.height
    }
    #[test]
    fn persistent_updates_match_ordered_stacks_and_preserve_checkpoints() {
        let mut tree = Instances::default();
        let mut model: BTreeMap<Key, Vec<ClassicalValue>> = BTreeMap::new();
        let mut rng = 17u64;
        let mut checkpoints = Vec::new();
        for i in 0..10000 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let key = (rng % 3, format!("x{}", (rng >> 16) % 127));
            if !(rng >> 32).is_multiple_of(3) {
                let value = ClassicalValue::Int(64, i);
                tree.push(key.clone(), value.clone());
                model.entry(key).or_default().push(value);
            } else {
                let exists = model.contains_key(&key);
                assert_eq!(tree.pop(&key), exists);
                if let Some(values) = model.get_mut(&key) {
                    values.pop().unwrap();
                    if values.is_empty() {
                        model.remove(&key);
                    }
                }
            }
            check(&tree);
            if i % 100 == 0 {
                checkpoints.push((tree.clone(), model.clone()));
            }
        }
        checkpoints.push((tree, model));
        for (tree, model) in checkpoints {
            assert_eq!(
                tree.keys().collect::<Vec<_>>(),
                model.keys().collect::<Vec<_>>()
            );
            for (key, values) in model {
                let mut stack = tree.get(&key).unwrap().clone();
                for value in values.iter().rev() {
                    assert_eq!(stack.last(), Some(value));
                    stack = stack.0.as_ref().unwrap().tail.clone();
                }
                assert_eq!(stack.last(), None);
            }
        }
    }
    #[test]
    fn long_shared_stacks_compare_and_drop_iteratively() {
        let key = (0, "loop".into());
        let mut tree = Instances::default();
        for _ in 0..40000 {
            tree.push(key.clone(), ClassicalValue::Int(8, 1));
        }
        let before = tree.clone();
        tree.push(key.clone(), ClassicalValue::Int(8, 2));
        assert_ne!(tree, before);
        assert!(tree.pop(&key));
        assert_eq!(tree, before);
        drop(before);
        drop(tree);
    }
}
