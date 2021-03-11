use atomic_polyfill::{AtomicU32, Ordering};

use crate::fmt::{assert, *};

pub struct AtomicBitset<const N: usize>
where
    [AtomicU32; (N + 31) / 32]: Sized,
{
    used: [AtomicU32; (N + 31) / 32],
}

impl<const N: usize> AtomicBitset<N>
where
    [AtomicU32; (N + 31) / 32]: Sized,
{
    pub const fn new() -> Self {
        const Z: AtomicU32 = AtomicU32::new(0);
        Self {
            used: [Z; (N + 31) / 32],
        }
    }

    pub fn alloc(&self) -> Option<usize> {
        for (i, val) in self.used.iter().enumerate() {
            let res = val.fetch_update(Ordering::AcqRel, Ordering::Acquire, |val| {
                let n = val.trailing_ones() as usize + i * 32;
                if n >= N {
                    None
                } else {
                    Some(val | (1 << n))
                }
            });
            if let Ok(val) = res {
                let n = val.trailing_ones() as usize + i * 32;
                return Some(n);
            }
        }
        None
    }
    pub fn free(&self, i: usize) {
        assert!(i < N);
        self.used[i / 32].fetch_and(!(1 << ((i % 32) as u32)), Ordering::AcqRel);
    }
}
