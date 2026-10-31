//! Shared immutable retained buffers. Cloning never allocates or copies payloads.
use crate::{reserve_more, reserved, Result};
use std::{fmt, ops::Deref, sync::Arc};

#[derive(Clone)]
pub struct Retained<T> {
    data: Arc<Vec<T>>,
    len: usize,
}
impl<T> Default for Retained<T> {
    fn default() -> Self {
        Self::from(Vec::new())
    }
}
impl<T> From<Vec<T>> for Retained<T> {
    fn from(data: Vec<T>) -> Self {
        let len = data.len();
        // Only the fixed-size owner is allocated here; the reserved Vec moves
        // into it. Arc<[T]>::from(Vec<T>) would allocate/copy the payload again.
        Self {
            data: Arc::new(data),
            len,
        }
    }
}
impl<T> Deref for Retained<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.data[..self.len]
    }
}
impl<T: PartialEq, U: AsRef<[T]>> PartialEq<U> for Retained<T> {
    fn eq(&self, other: &U) -> bool {
        self.deref() == other.as_ref()
    }
}
impl<T: Eq> Eq for Retained<T> {}
impl<T> AsRef<[T]> for Retained<T> {
    fn as_ref(&self) -> &[T] {
        self
    }
}
impl<T> fmt::Debug for Retained<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Retained")
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}
impl<T: Clone> Retained<T> {
    /// Prepare a unique, sufficiently large vector before visible mutation.
    pub(crate) fn prepare(&mut self, extra: usize, rule: &str, what: &str) -> Result<&mut Vec<T>> {
        if Arc::get_mut(&mut self.data).is_none() || self.len != self.data.len() {
            let total = self.len.checked_add(extra).ok_or_else(|| {
                crate::resource_error(rule, format!("{what} cumulative count exceeds usize"))
            })?;
            let mut copy = reserved(total, rule, what)?;
            copy.extend_from_slice(self);
            self.data = Arc::new(copy);
        } else {
            reserve_more(
                Arc::get_mut(&mut self.data).expect("unique buffer"),
                extra,
                rule,
                what,
            )?;
        }
        self.len += extra;
        Ok(Arc::get_mut(&mut self.data).expect("prepared unique buffer"))
    }
    pub(crate) fn truncate(&mut self, len: usize) {
        self.len = self.len.min(len);
    }
}

/// Sparse byte map with shared snapshots and fallible copy-on-write preparation.
/// Sorted resident entries; virtual allocation lengths never expand this map.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SparseBytes(Retained<(u128, u8)>);
impl SparseBytes {
    pub fn get(&self, address: &u128) -> Option<&u8> {
        self.0
            .binary_search_by_key(address, |e| e.0)
            .ok()
            .map(|i| &self.0[i].1)
    }
    pub fn keys(&self) -> impl Iterator<Item = &u128> {
        self.0.iter().map(|e| &e.0)
    }
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn prepare(&mut self, extra: usize) -> Result<()> {
        // Reserve geometric growth when unique, to keep byte-at-a-time stores
        // amortized. Snapshot COW still reserves the complete copy fallibly.
        let data = self
            .0
            .prepare(0, "§7.1/§7.4/§10", "sparse memory retention")?;
        data.try_reserve(extra).map_err(|e| {
            crate::resource_error("§7.1/§7.4/§10", format!("sparse memory retention: {e}"))
        })
    }
    pub fn set(&mut self, address: u128, byte: u8) -> Result<()> {
        let found = self.0.binary_search_by_key(&address, |e| e.0);
        if (byte == 0 && found.is_err()) || found.is_ok_and(|i| self.0[i].1 == byte) {
            return Ok(());
        }
        self.prepare(usize::from(byte != 0 && found.is_err()))?;
        let data = Arc::get_mut(&mut self.0.data).expect("prepared sparse memory");
        match (found, byte) {
            (Ok(i), 0) => {
                data.remove(i);
            }
            (Ok(i), _) => data[i].1 = byte,
            (Err(i), _) => data.insert(i, (address, byte)),
        }
        self.0.len = data.len();
        Ok(())
    }
}
impl<'a> IntoIterator for &'a SparseBytes {
    type Item = &'a (u128, u8);
    type IntoIter = std::slice::Iter<'a, (u128, u8)>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn r22_copy_on_write_retention_memory_limited() {
        if std::env::var_os("R22_COW_CHILD").is_none() {
            let out = std::process::Command::new("python3")
                .args(["-c", "import os,resource,sys; resource.setrlimit(resource.RLIMIT_AS,(128*1024*1024,)*2); resource.setrlimit(resource.RLIMIT_CORE,(0,0)); os.execv(sys.argv[1],sys.argv[1:])"])
                .arg(std::env::current_exe().unwrap())
                .args(["--exact", "retained::tests::r22_copy_on_write_retention_memory_limited", "--nocapture"])
                .env("R22_COW_CHILD", "1").env("MALLOC_ARENA_MAX", "1").output().unwrap();
            assert!(out.status.success(), "{out:?}");
            return;
        }
        for len in [0, 1, 8, 1 << 20, 64 << 20] {
            let mut bytes = reserved(len, "§10", "first buffer").unwrap();
            bytes.resize(len, 0u8);
            let a = Retained::from(bytes);
            let mut b = a.clone();
            assert_eq!(a.as_ptr(), b.as_ptr());
            let r = b.prepare(1, "§7.4/§10", "copy-on-write retention");
            if len == 64 << 20 {
                let e = r.unwrap_err();
                assert_eq!(e.kind(), crate::ErrorKind::HostResource);
                assert!(e.to_string().contains("copy-on-write retention"));
                assert_eq!(a.as_ptr(), b.as_ptr());
                assert_eq!(a, b);
            } else {
                r.unwrap().push(1);
                assert_eq!(a.len(), len);
                assert_eq!(b.len(), len + 1);
                assert_eq!(b[len], 1);
                b.truncate(len);
                assert_eq!(a, b);
            }
        }
    }
}
