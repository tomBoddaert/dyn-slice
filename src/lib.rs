//! An implementation for a `&[dyn Trait]`-like type, inspired by a [Reddit thread](https://www.reddit.com/r/rust/comments/14i08gz/dyn_slices).
//!
//! Indexing into a dyn-slice yields a dyn object.
//!
//! # Examples
//!
//! ```
//! use dyn_slice::DynSlice;
//! use std::fmt::Display;
//!
//! let array: [u8; 4] = [1, 2, 3, 4];
//! // Create the first dyn slice
//! let slice = DynSlice::<dyn Display>::new(&array);
//!
//! let array2: [i16; 3] = [5, 6, 7];
//! // Create the second dyn slice
//! let slice2 = DynSlice::<dyn Display>::new(&array2);
//!
//! // The iterators can be chained because they are iterators
//! // over `&dyn Display` rather than over the underlying types
//! let iter = slice.iter().chain(slice2.iter());
//! for n in iter {
//!     println!("{n}");
//! }
//! ```
//!
#![doc = concat!("There are more examples in the [examples directory](https://docs.rs/crate/dyn-slice/", env!("CARGO_PKG_VERSION"), "/source/examples/).")]
#![feature(
    ptr_metadata,
    unsize,
    reborrow,
    iter_advance_by,
    trusted_len,
    const_trait_impl,
    const_clone,
    const_iter,
    const_destruct
)]
#![cfg_attr(not(feature = "std"), no_std)]

use core::{
    fmt,
    marker::{PhantomData, Unsize},
    ops::{CoerceShared, Index, IndexMut, Reborrow},
    ptr::{self, DynMetadata, NonNull, Pointee},
};

// mod as_slice;
mod index;
pub mod iter;
mod standard;
pub use index::SliceIndex;
#[cfg(test)]
mod compile_tests;

/// `NonNull<[dyn Trait]>`
pub struct DynSlicePtr<Dyn>
where
    Dyn: ?Sized,
{
    pub addr: NonNull<()>,
    pub len: usize,
    pub dyn_metadata: <Dyn as Pointee>::Metadata,
}

/// `&[dyn Trait]`
///
/// A type erased slice of elements that implement a trait.
///
/// # Example
/// ```
/// # use std::fmt::Debug;
/// # use dyn_slice::DynSlice;
/// #
/// let slice = DynSlice::<dyn Debug>::new(&[1, 2, 3, 4, 5]);
/// # assert_eq!(&format!("{slice:?}"), "[1, 2, 3, 4, 5]");
/// println!("{slice:?}"); // [1, 2, 3, 4, 5]
/// ```
#[repr(transparent)]
pub struct DynSlice<'slice, Dyn>
where
    Dyn: ?Sized,
{
    ptr: DynSlicePtr<Dyn>,
    _phantom: PhantomData<&'slice Dyn>,
}

/// `&mut [dyn Trait]`
///
/// A mutable type erased slice of elements that implement a trait.
///
/// # Example
/// ```
/// # use std::ops::AddAssign;
/// # use dyn_slice::DynSliceMut;
/// #
/// let mut array = [1, 2, 3, 4, 5];
/// let mut slice = DynSliceMut::<dyn AddAssign<_>>::new(&mut array);
/// slice.iter_mut().for_each(|x| *x += 10);
/// assert_eq!(array, [11, 12, 13, 14, 15]);
/// ```
#[repr(transparent)]
pub struct DynSliceMut<'slice, Dyn>
where
    Dyn: ?Sized,
{
    ptr: DynSlicePtr<Dyn>,
    _phantom: PhantomData<&'slice mut Dyn>,
}

impl<Dyn> DynSlicePtr<Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    // TODO: safety
    // - Must be in bounds of allocation
    // - Metadata must be valid
    #[must_use]
    #[inline]
    pub unsafe fn add(self, count: usize) -> NonNull<()> {
        let size = self.dyn_metadata.size_of();
        unsafe { self.addr.byte_add(count * size) }
    }

    #[must_use]
    #[inline]
    pub const unsafe fn as_ref<'slice>(self) -> DynSlice<'slice, Dyn> {
        DynSlice {
            ptr: self,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    pub const unsafe fn as_mut<'slice>(self) -> DynSliceMut<'slice, Dyn> {
        DynSliceMut {
            ptr: self,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    pub unsafe fn split_at_unchecked(self, mid: usize) -> (Self, Self) {
        debug_assert!(mid <= self.len);

        let left = Self {
            addr: self.addr,
            len: mid,
            dyn_metadata: self.dyn_metadata,
        };
        let right = Self {
            addr: unsafe { self.add(mid) },
            len: unsafe { self.len.unchecked_sub(mid) },
            dyn_metadata: self.dyn_metadata,
        };

        (left, right)
    }
}

impl<Dyn> fmt::Debug for DynSlicePtr<Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DynSlicePtr")
            .field("addr", &self.addr)
            .field("len", &self.len)
            .field("dyn_metadata", &self.dyn_metadata)
            .finish()
    }
}

impl<Dyn> const Clone for DynSlicePtr<Dyn>
where
    Dyn: ?Sized + Pointee,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<Dyn> Copy for DynSlicePtr<Dyn> where Dyn: ?Sized + Pointee {}

impl<'slice, Dyn> DynSlice<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[must_use]
    pub const fn new<T>(slice: &'slice [T]) -> Self
    where
        T: Unsize<Dyn> + 'slice,
    {
        let (ptr, len) = NonNull::from_ref(slice).to_raw_parts();
        let metadata = ptr::metadata(ptr::null::<T>() as *const Dyn);

        Self {
            ptr: DynSlicePtr {
                addr: ptr,
                len,
                dyn_metadata: metadata,
            },
            _phantom: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.ptr.len
    }

    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.ptr.len == 0
    }

    #[must_use]
    #[inline]
    pub const fn as_ptr(self) -> DynSlicePtr<Dyn> {
        self.ptr
    }

    #[must_use]
    #[inline]
    pub fn get<I>(self, index: I) -> Option<I::Output>
    where
        I: SliceIndex<Self>,
    {
        index.get(self)
    }

    #[must_use]
    pub const fn first(self) -> Option<&'slice Dyn> {
        if self.is_empty() {
            return None;
        }

        let ptr = NonNull::from_raw_parts(self.ptr.addr, self.ptr.dyn_metadata);

        Some(unsafe { ptr.as_ref() })
    }

    #[must_use]
    pub fn last(self) -> Option<&'slice Dyn> {
        let i = self.ptr.len.checked_sub(1)?;
        let ptr = unsafe { self.ptr.add(i) };

        let ptr = NonNull::from_raw_parts(ptr, self.ptr.dyn_metadata);

        Some(unsafe { ptr.as_ref() })
    }

    #[must_use]
    pub unsafe fn split_at_unchecked(self, mid: usize) -> (Self, Self) {
        let (left, right) = unsafe { self.ptr.split_at_unchecked(mid) };
        (unsafe { left.as_ref() }, unsafe { right.as_ref() })
    }

    #[must_use]
    pub fn split_at_checked(self, mid: usize) -> Option<(Self, Self)> {
        (mid <= self.ptr.len).then(|| unsafe { self.split_at_unchecked(mid) })
    }

    #[must_use]
    pub fn split_at(self, mid: usize) -> (Self, Self) {
        if let Some(pair) = self.split_at_checked(mid) {
            pair
        } else {
            panic!("mid > len")
        }
    }

    #[must_use]
    #[inline]
    pub const fn iter(self) -> iter::Iter<'slice, Dyn> {
        self.into_iter()
    }

    #[must_use]
    #[inline]
    pub const fn split<P>(self, predicate: P) -> iter::Split<'slice, Dyn, P>
    where
        P: FnMut(&Dyn) -> bool,
    {
        iter::Split::new(self, predicate)
    }
}

impl<Dyn> const Clone for DynSlice<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}
impl<Dyn> Copy for DynSlice<'_, Dyn> where Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> {}

impl<Dyn> fmt::Pointer for DynSlice<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.ptr, f)
    }
}

impl<Dyn> Index<usize> for DynSlice<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = Dyn;

    fn index(&self, index: usize) -> &Self::Output {
        // TODO: use SliceIndex::index
        self.get(index).expect("index out of range")
    }
}

impl<'slice, Dyn> DynSliceMut<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[must_use]
    pub const fn new<T>(slice: &'slice mut [T]) -> Self
    where
        T: Unsize<Dyn> + 'slice,
    {
        let (ptr, len) = NonNull::from_mut(slice).to_raw_parts();
        let metadata = ptr::metadata(ptr::null_mut::<T>() as *mut Dyn);

        Self {
            ptr: DynSlicePtr {
                addr: ptr,
                len,
                dyn_metadata: metadata,
            },
            _phantom: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.ptr.len
    }

    #[must_use]
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.ptr.len == 0
    }

    #[must_use]
    #[inline]
    pub const fn as_ptr(self) -> DynSlicePtr<Dyn> {
        self.ptr
    }

    #[must_use]
    #[inline]
    pub const fn reborrow(&mut self) -> DynSliceMut<'_, Dyn> {
        unsafe { self.ptr.as_mut() }
    }
    #[must_use]
    #[inline]
    pub const fn coerce_shared(&self) -> DynSlice<'_, Dyn> {
        unsafe { self.ptr.as_ref() }
    }

    #[must_use]
    #[inline]
    pub fn get_mut<'a, I>(&'a mut self, index: I) -> Option<I::Output>
    where
        I: SliceIndex<DynSliceMut<'a, Dyn>>,
    {
        index.get(self.reborrow())
    }

    #[must_use]
    pub const fn first_mut(&mut self) -> Option<&mut Dyn> {
        if self.is_empty() {
            return None;
        }

        let mut ptr = NonNull::from_raw_parts(self.ptr.addr, self.ptr.dyn_metadata);

        Some(unsafe { ptr.as_mut() })
    }

    #[must_use]
    pub fn last_mut(&mut self) -> Option<&mut Dyn> {
        let i = self.ptr.len.checked_sub(1)?;
        let ptr = unsafe { self.ptr.add(i) };

        let mut ptr = NonNull::from_raw_parts(ptr, self.ptr.dyn_metadata);

        Some(unsafe { ptr.as_mut() })
    }

    #[must_use]
    pub unsafe fn split_at_mut_unchecked(self, mid: usize) -> (Self, Self) {
        let (left, right) = unsafe { self.ptr.split_at_unchecked(mid) };
        (unsafe { left.as_mut() }, unsafe { right.as_mut() })
    }

    #[must_use]
    pub fn split_at_mut_checked(self, mid: usize) -> Option<(Self, Self)> {
        (mid <= self.ptr.len).then(|| unsafe { self.split_at_mut_unchecked(mid) })
    }

    #[must_use]
    pub fn split_at_mut(self, mid: usize) -> (Self, Self) {
        if let Some(pair) = self.split_at_mut_checked(mid) {
            pair
        } else {
            panic!("mid > len")
        }
    }

    #[must_use]
    #[inline]
    pub const fn iter_mut(self) -> iter::IterMut<'slice, Dyn> {
        self.into_iter()
    }

    #[must_use]
    #[inline]
    pub const fn split_mut<P>(self, predicate: P) -> iter::SplitMut<'slice, Dyn, P>
    where
        P: FnMut(&Dyn) -> bool,
    {
        iter::SplitMut::new(self, predicate)
    }
}

impl<Dyn> Reborrow for DynSliceMut<'_, Dyn> where Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> {}
impl<'slice, Dyn> CoerceShared for DynSliceMut<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Target = DynSlice<'slice, Dyn>;
}

impl<Dyn> fmt::Pointer for DynSliceMut<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&self.coerce_shared(), f)
    }
}

impl<Dyn> Index<usize> for DynSliceMut<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = Dyn;

    fn index(&self, index: usize) -> &Self::Output {
        // TODO: use SliceIndex::index
        self.coerce_shared().get(index).expect("index out of range")
    }
}
impl<Dyn> IndexMut<usize> for DynSliceMut<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        // TODO: use SliceIndex::index
        self.get_mut(index).expect("index out of range")
    }
}

#[cfg(test)]
mod test {
    use core::borrow::BorrowMut;

    use crate::{DynSlice, DynSliceMut};

    trait Tr: PartialEq<u8> + BorrowMut<u8> + core::fmt::Debug {}
    impl<T> Tr for T where T: PartialEq<u8> + BorrowMut<u8> + core::fmt::Debug {}

    static VE: [u8; 10] = [0, 0, 1, 0, 2, 0, 2, 2, 1, 6];
    static VE2: [u8; 10] = [0, 5, 0, 2, 6, 5, 4, 0, 5, 3];

    #[test]
    fn new() {
        let slice = DynSlice::<dyn Tr>::new(&VE);

        assert_eq!(slice.ptr.len, VE.len());
        assert_eq!(slice.ptr.addr.addr().get(), (&raw const VE).addr());
    }

    #[test]
    fn new_mut() {
        let mut array = VE;
        let slice = DynSliceMut::<dyn Tr>::new(&mut array);

        assert_eq!(slice.ptr.len, VE.len());
        assert_eq!(slice.ptr.addr.addr().get(), (&raw const array).addr());
    }

    #[test]
    fn get() {
        let slice = DynSlice::<dyn Tr>::new(&VE);

        for (i, n) in VE.iter().enumerate() {
            let e = slice.get(i).unwrap();
            assert_eq!(e, n);
        }

        assert!(slice.get(VE.len()).is_none());
    }

    #[test]
    fn get_mut() {
        let mut array = VE;
        let mut slice = DynSliceMut::<dyn Tr>::new(&mut array);

        for (i, (n, m)) in VE.iter().zip(VE2).enumerate() {
            let e = slice.get_mut(i).unwrap();
            assert_eq!(e, n);

            *e.borrow_mut() = m;
        }

        assert!(slice.get_mut(VE.len()).is_none());

        assert_eq!(array, VE2);
    }

    #[test]
    fn debug() {
        let slice = DynSlice::<dyn core::fmt::Debug>::new(&VE);
        assert_eq!(format!("{slice:?}"), format!("{VE:?}"));
    }
}
