#![cfg_attr(not(test), no_std)]

mod atomic_bitset;

use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicU32;
use core::{cmp, mem, ptr::NonNull};

use crate::atomic_bitset::AtomicBitset;

/// Implementation detail. Not covered by semver guarantees.
#[doc(hidden)]
pub trait PoolStorage<T> {
    fn alloc(&self) -> Option<NonNull<T>>;
    unsafe fn free(&self, p: NonNull<T>);
}

/// Implementation detail. Not covered by semver guarantees.
#[doc(hidden)]
pub struct PoolStorageImpl<T, const N: usize, const K: usize>
where
    [AtomicU32; K]: Sized,
{
    used: AtomicBitset<N, K>,
    data: MaybeUninit<[T; N]>,
}

impl<T, const N: usize, const K: usize> PoolStorageImpl<T, N, K>
where
    [AtomicU32; K]: Sized,
{
    pub const fn new() -> Self {
        Self {
            used: AtomicBitset::new(),
            data: MaybeUninit::uninit(),
        }
    }
}

impl<T, const N: usize, const K: usize> PoolStorage<T> for PoolStorageImpl<T, N, K>
where
    [AtomicU32; K]: Sized,
{
    fn alloc(&self) -> Option<NonNull<T>> {
        let n = self.used.alloc()?;
        let origin = self.data.as_ptr() as *mut T;
        Some(unsafe { NonNull::new_unchecked(origin.add(n)) })
    }

    /// safety: p must be a pointer obtained from self.alloc that hasn't been freed yet.
    unsafe fn free(&self, p: NonNull<T>) {
        let origin = self.data.as_ptr() as *mut T;
        let n = p.as_ptr().offset_from(origin);
        assert!(n >= 0);
        assert!((n as usize) < N);
        self.used.free(n as usize);
    }
}

pub trait Pool: 'static {
    type Item: 'static;

    /// Implementation detail. Not covered by semver guarantees.
    #[doc(hidden)]
    type Storage: PoolStorage<Self::Item>;

    /// Implementation detail. Not covered by semver guarantees.
    #[doc(hidden)]
    fn get() -> &'static Self::Storage;
}

pub struct Box<P: Pool> {
    ptr: NonNull<P::Item>,
}

impl<P: Pool> Box<P> {
    pub fn new(item: P::Item) -> Option<Self> {
        let p = match P::get().alloc() {
            Some(p) => p,
            None => return None,
        };
        unsafe { p.as_ptr().write(item) };
        Some(Self { ptr: p })
    }

    pub fn into_raw(b: Self) -> NonNull<P::Item> {
        let res = b.ptr;
        mem::forget(b);
        res
    }

    pub unsafe fn from_raw(ptr: NonNull<P::Item>) -> Self {
        Self { ptr }
    }
}

impl<P: Pool> Drop for Box<P> {
    fn drop(&mut self) {
        unsafe {
            //trace!("dropping {:u32}", self.ptr as u32);
            self.ptr.as_ptr().drop_in_place();
            P::get().free(self.ptr);
        };
    }
}

unsafe impl<P: Pool> Send for Box<P> where P::Item: Send {}

unsafe impl<P: Pool> Sync for Box<P> where P::Item: Sync {}

unsafe impl<P: Pool> stable_deref_trait::StableDeref for Box<P> {}

impl<P: Pool> as_slice_01::AsSlice for Box<P>
where
    P::Item: as_slice_01::AsSlice,
{
    type Element = <P::Item as as_slice_01::AsSlice>::Element;

    fn as_slice(&self) -> &[Self::Element] {
        self.deref().as_slice()
    }
}

impl<P: Pool> as_slice_01::AsMutSlice for Box<P>
where
    P::Item: as_slice_01::AsMutSlice,
{
    fn as_mut_slice(&mut self) -> &mut [Self::Element] {
        self.deref_mut().as_mut_slice()
    }
}

impl<P: Pool> as_slice_02::AsSlice for Box<P>
where
    P::Item: as_slice_02::AsSlice,
{
    type Element = <P::Item as as_slice_02::AsSlice>::Element;

    fn as_slice(&self) -> &[Self::Element] {
        self.deref().as_slice()
    }
}

impl<P: Pool> as_slice_02::AsMutSlice for Box<P>
where
    P::Item: as_slice_02::AsMutSlice,
{
    fn as_mut_slice(&mut self) -> &mut [Self::Element] {
        self.deref_mut().as_mut_slice()
    }
}

impl<P: Pool> Deref for Box<P> {
    type Target = P::Item;

    fn deref(&self) -> &P::Item {
        unsafe { self.ptr.as_ref() }
    }
}

impl<P: Pool> DerefMut for Box<P> {
    fn deref_mut(&mut self) -> &mut P::Item {
        unsafe { self.ptr.as_mut() }
    }
}

impl<P: Pool> core::fmt::Debug for Box<P>
where
    P::Item: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <P::Item as core::fmt::Debug>::fmt(self, f)
    }
}

impl<P: Pool> core::fmt::Display for Box<P>
where
    P::Item: core::fmt::Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <P::Item as core::fmt::Display>::fmt(self, f)
    }
}

impl<P: Pool> PartialEq for Box<P>
where
    P::Item: PartialEq,
{
    fn eq(&self, rhs: &Box<P>) -> bool {
        <P::Item as PartialEq>::eq(self, rhs)
    }
}

impl<P: Pool> Eq for Box<P> where P::Item: Eq {}

impl<P: Pool> PartialOrd for Box<P>
where
    P::Item: PartialOrd,
{
    fn partial_cmp(&self, rhs: &Box<P>) -> Option<cmp::Ordering> {
        <P::Item as PartialOrd>::partial_cmp(self, rhs)
    }
}

impl<P: Pool> Ord for Box<P>
where
    P::Item: Ord,
{
    fn cmp(&self, rhs: &Box<P>) -> cmp::Ordering {
        <P::Item as Ord>::cmp(self, rhs)
    }
}

impl<P: Pool> Hash for Box<P>
where
    P::Item: Hash,
{
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        <P::Item as Hash>::hash(self, state)
    }
}

#[macro_export]
macro_rules! pool {
    ($vis:vis $name:ident: [$ty:ty; $n:expr]) => {
        $vis struct $name { _uninhabited: ::core::convert::Infallible }
        impl $crate::Pool for $name {
            type Item = $ty;
            type Storage = $crate::PoolStorageImpl<$ty, {$n}, {($n+31)/32}>;
            fn get() -> &'static Self::Storage {
                static POOL: $crate::PoolStorageImpl<$ty, {$n}, {($n+31)/32}> = $crate::PoolStorageImpl::new();
                &POOL
            }
        }
    };
}
#[cfg(test)]
mod test {
    use super::*;

    pool!(TestPool: [u32; 4]);

    #[test]
    fn test_pool() {
        let b1 = Box::<TestPool>::new(111).unwrap();
        let b2 = Box::<TestPool>::new(222).unwrap();
        let b3 = Box::<TestPool>::new(333).unwrap();
        let b4 = Box::<TestPool>::new(444).unwrap();
        assert!(Box::<TestPool>::new(555).is_none());
        assert_eq!(*b1, 111);
        assert_eq!(*b2, 222);
        assert_eq!(*b3, 333);
        assert_eq!(*b4, 444);
        mem::drop(b3);
        let b5 = Box::<TestPool>::new(555).unwrap();
        assert!(Box::<TestPool>::new(666).is_none());
        assert_eq!(*b1, 111);
        assert_eq!(*b2, 222);
        assert_eq!(*b4, 444);
        assert_eq!(*b5, 555);
    }
}
