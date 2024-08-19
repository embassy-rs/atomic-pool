use super::AtomicBitset;

/// Automatically frees the Bitset slot when DroppableBit is dropped
/// Useful for async environments where the future might be dropped before it completes
pub struct DroppableBit<'a, const N: usize, const K: usize> {
    bitset: &'a AtomicBitset<N, K>,
    inner: usize,
}

impl<'a, const N: usize, const K: usize> DroppableBit<'a, N, K> {
    /// Only a single instance of DroppableBit should be created for each slot
    /// Restrict it to only be created by AtomicBitset `alloc_droppable` method
    pub(super) fn new(bitset: &'a AtomicBitset<N, K>, inner: usize) -> Self {
        Self { bitset, inner }
    }

    pub fn inner(&self) -> usize {
        self.inner
    }
}

impl<const N: usize, const K: usize> Drop for DroppableBit<'_, N, K> {
    fn drop(&mut self) {
        self.bitset.free(self.inner);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_16() {
        let s = AtomicBitset::<16, 1>::new();
        let mut v = vec![];

        for _ in 0..16 {
            let bit = s.alloc().map(|i| DroppableBit::new(&s, i));
            assert!(bit.is_some());

            v.push(bit.unwrap());
        }
        assert_eq!(s.alloc(), None);
        v.pop();
        v.pop();
        assert!(s.alloc().is_some());
        assert!(s.alloc().is_some());
        assert_eq!(s.alloc(), None);
        v.pop();
        assert!(s.alloc().is_some());
    }
}