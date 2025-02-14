use portable_atomic::{AtomicU32, Ordering};

use crate::ATOMICU32_BITS;

pub struct AtomicBitset<const N: usize, const K: usize>
where
    [AtomicU32; K]: Sized,
{
    used: [AtomicU32; K],
}

impl<const N: usize, const K: usize> AtomicBitset<N, K>
where
    [AtomicU32; K]: Sized,
{
    pub const fn new() -> Self {
        Self {
            used: [const { AtomicU32::new(0) }; K],
        }
    }

    pub fn alloc(&self) -> Option<usize> {
        for (i, val) in self.used.iter().enumerate() {
            let mut allocated = 0;
            let res = val.fetch_update(Ordering::AcqRel, Ordering::Acquire, |val| {
                let zero_bit = val.trailing_ones() as usize;
                let maybe_allocated = zero_bit + i * ATOMICU32_BITS;

                if zero_bit == ATOMICU32_BITS {
                    // there are no zero bits
                    None
                } else if maybe_allocated >= N {
                    // there are zero bits, but this is the last AtomicU32 in the array, and the only
                    // zero bits left are out of range.
                    None
                } else {
                    // There's a zero bit in range! set it to 1.
                    allocated = maybe_allocated;
                    Some(val | (1 << zero_bit))
                }
            });
            if res.is_ok() {
                return Some(allocated);
            }
        }
        None
    }
    pub fn free(&self, i: usize) {
        assert!(i < N);
        self.used[i / ATOMICU32_BITS]
            .fetch_and(!(1 << ((i % ATOMICU32_BITS) as u32)), Ordering::AcqRel);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_16() {
        let s = AtomicBitset::<16, 1>::new();
        for i in 0..16 {
            assert_eq!(s.alloc(), Some(i));
        }
        assert_eq!(s.alloc(), None);
        s.free(2);
        s.free(8);
        s.free(4);
        assert_eq!(s.alloc(), Some(2));
        assert_eq!(s.alloc(), Some(4));
        s.free(3);
        assert_eq!(s.alloc(), Some(3));
        assert_eq!(s.alloc(), Some(8));
        assert_eq!(s.alloc(), None);
    }

    #[test]
    fn test_32() {
        let s = AtomicBitset::<32, 1>::new();
        for i in 0..32 {
            assert_eq!(s.alloc(), Some(i));
        }
        assert_eq!(s.alloc(), None);
        s.free(2);
        s.free(31);
        s.free(0);
        assert_eq!(s.alloc(), Some(0));
        assert_eq!(s.alloc(), Some(2));
        s.free(3);
        assert_eq!(s.alloc(), Some(3));
        assert_eq!(s.alloc(), Some(31));
        assert_eq!(s.alloc(), None);
    }

    #[test]
    fn test_48() {
        let s = AtomicBitset::<48, 2>::new();
        for i in 0..48 {
            assert_eq!(s.alloc(), Some(i));
        }
        assert_eq!(s.alloc(), None);
        s.free(2);
        s.free(46);
        s.free(4);
        s.free(47);
        assert_eq!(s.alloc(), Some(2));
        assert_eq!(s.alloc(), Some(4));
        s.free(3);
        assert_eq!(s.alloc(), Some(3));
        assert_eq!(s.alloc(), Some(46));
        assert_eq!(s.alloc(), Some(47));
        assert_eq!(s.alloc(), None);
    }
    #[test]
    fn test_64() {
        let s = AtomicBitset::<64, 2>::new();
        for i in 0..64 {
            assert_eq!(s.alloc(), Some(i));
        }
        assert_eq!(s.alloc(), None);
        s.free(2);
        s.free(46);
        s.free(61);
        s.free(4);
        s.free(47);
        assert_eq!(s.alloc(), Some(2));
        assert_eq!(s.alloc(), Some(4));
        s.free(3);
        assert_eq!(s.alloc(), Some(3));
        assert_eq!(s.alloc(), Some(46));
        assert_eq!(s.alloc(), Some(47));
        assert_eq!(s.alloc(), Some(61));
        assert_eq!(s.alloc(), None);
    }

    #[cfg(not(miri))] // too slow
    #[test]
    fn test_31337() {
        let s = AtomicBitset::<31337, 980>::new();
        for i in 0..31337 {
            assert_eq!(s.alloc(), Some(i));
        }
        assert_eq!(s.alloc(), None);
        s.free(26123);
        s.free(6423);
        s.free(4241);
        s.free(47);
        assert_eq!(s.alloc(), Some(47));
        assert_eq!(s.alloc(), Some(4241));
        s.free(3);
        assert_eq!(s.alloc(), Some(3));
        assert_eq!(s.alloc(), Some(6423));
        assert_eq!(s.alloc(), Some(26123));
        assert_eq!(s.alloc(), None);
    }
}
