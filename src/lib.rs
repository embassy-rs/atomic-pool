#![no_std]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]
#![allow(incomplete_features)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod atomic_bitset;

use as_slice::{AsMutSlice, AsSlice};
use core::{cmp, mem, ptr::NonNull};
use core::hash::{Hash, Hasher};
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicU32;

use crate::atomic_bitset::AtomicBitset;
use crate::fmt::{assert, *};

pub trait PoolStorage<T> {
    fn alloc(&self) -> Option<NonNull<T>>;
    unsafe fn free(&self, p: NonNull<T>);
}

pub struct PoolStorageImpl<T, const N: usize>
where
    [AtomicU32; (N + 31) / 32]: Sized,
{
    used: AtomicBitset<N>,
    data: MaybeUninit<[T; N]>,
}

impl<T, const N: usize> PoolStorageImpl<T, N>
where
    [AtomicU32; (N + 31) / 32]: Sized,
{
    pub const fn new() -> Self {
        Self {
            used: AtomicBitset::new(),
            data: MaybeUninit::uninit(),
        }
    }
}

impl<T, const N: usize> PoolStorage<T> for PoolStorageImpl<T, N>
where
    [AtomicU32; (N + 31) / 32]: Sized,
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
    type Storage: PoolStorage<Self::Item>;
    fn get() -> &'static Self::Storage;
}

pub struct Box<P: Pool> {
    ptr: NonNull<P::Item>,
}

impl<P: Pool> Box<P> {
    pub fn new(item: P::Item) -> Option<Self> {
        let p = match P::get().alloc() {
            Some(p) => p,
            None => {
                warn!("alloc failed!");
                return None;
            }
        };
        //trace!("allocated {:u32}", p as u32);
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

impl<P: Pool> AsSlice for Box<P>
where
    P::Item: AsSlice,
{
    type Element = <P::Item as AsSlice>::Element;

    fn as_slice(&self) -> &[Self::Element] {
        self.deref().as_slice()
    }
}

impl<P: Pool> AsMutSlice for Box<P>
where
    P::Item: AsMutSlice,
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
    ($vis:vis $name:ident: [$ty:ty; $size:expr]) => {
        $vis struct $name;
        impl $crate::Pool for $name {
            type Item = $ty;
            type Storage = $crate::PoolStorageImpl<$ty, {$size}>;
            fn get() -> &'static Self::Storage {
                static POOL: $crate::PoolStorageImpl<$ty, {$size}> = $crate::PoolStorageImpl::new();
                &POOL
            }
        }
    };
}
