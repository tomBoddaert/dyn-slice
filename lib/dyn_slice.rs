use core::{
    marker::PhantomData,
    mem::transmute,
    ops::{Bound, Index, RangeBounds},
    ptr,
    ptr::{DynMetadata, Pointee},
    slice,
};

use crate::Iter;

/// `&dyn [Trait]`
pub struct DynSlice<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> {
    pub(crate) vtable_ptr: *const (),
    pub(crate) len: usize,
    pub(crate) data: *const (),
    phantom: PhantomData<&'a Dyn>,
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> Clone for DynSlice<'a, Dyn> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> Copy for DynSlice<'a, Dyn> {}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> DynSlice<'a, Dyn> {
    #[inline]
    #[must_use]
    /// Construct a dyn slice given a slice and a vtable pointer.
    ///
    /// # Safety
    /// Caller must ensure that `vtable_ptr` is a valid instance of `DynMetadata` for `DynSliceFromType` and `Dyn` transmuted, or optionally, a null pointer if `value.len() == 0`.
    pub const unsafe fn with_vtable_ptr<DynSliceFromType>(
        value: &'a [DynSliceFromType],
        vtable_ptr: *const (),
    ) -> Self {
        Self {
            vtable_ptr,
            len: value.len(),
            data: value.as_ptr().cast(),
            phantom: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    /// Construct a dyn slice given a slice and a `DynMetadata` instance.
    ///
    /// # Safety
    /// Caller must ensure that `metadata` is a valid instance of `DynMetadata` for `DynSliceFromType` and `Dyn`.
    pub const unsafe fn with_metadata<DynSliceFromType>(
        value: &'a [DynSliceFromType],
        metadata: DynMetadata<Dyn>,
    ) -> Self {
        Self::with_vtable_ptr(value, transmute(metadata))
    }

    #[inline]
    #[must_use]
    /// Construct a dyn slice from raw parts.
    ///
    /// # Safety
    /// Caller must ensure that:
    /// - `vtable_ptr` is a valid instance of `DynMetadata` transmuted, or optionally, a null pointer if `len == 0`,
    /// - `len` <= then length of the slice in memory from the `data` pointer,
    /// - `data` is a valid pointer to the slice,
    /// - the underlying slice is the same layout as [`[T]`](https://doc.rust-lang.org/reference/type-layout.html#slice-layout)
    pub const unsafe fn from_parts(vtable_ptr: *const (), len: usize, data: *const ()) -> Self {
        Self {
            vtable_ptr,
            len,
            data,
            phantom: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    /// Construct a dyn slice from raw parts with a `DynMetadata` instance rather than a vtable pointer.
    ///
    /// # Safety
    /// Caller must ensure that:
    /// - `metadata` is a valid instance of `DynMetadata`,
    /// - `len` <= then length of the slice in memory from the `data` pointer,
    /// - `data` is a valid pointer to the slice,
    /// - the underlying slice is the same layout as [`[T]`](https://doc.rust-lang.org/reference/type-layout.html#slice-layout)
    pub unsafe fn from_parts_with_metadata(
        metadata: DynMetadata<Dyn>,
        len: usize,
        data: *const (),
    ) -> Self {
        Self::from_parts(transmute(metadata), len, data)
    }

    #[inline]
    #[must_use]
    /// Get the vtable pointer, which may be null if the slice is empty.
    pub const fn vtable_ptr(&self) -> *const () {
        self.vtable_ptr
    }

    #[inline]
    #[must_use]
    /// Get the metadata component of the element's pointers, or possibly `None` if the slice is empty.
    pub fn metadata(&self) -> Option<DynMetadata<Dyn>> {
        let vtable_ptr = self.vtable_ptr();
        (!vtable_ptr.is_null()).then(|| {
            // SAFETY:
            // DynMetadata only contains a single pointer, and has the same layout as *const ().
            // The statement above guarantees that the pointer is not null and so, the pointer is
            // guaranteed to point to a vtable by the safe methods that create the slice.
            unsafe { transmute(vtable_ptr) }
        })
    }

    #[inline]
    #[must_use]
    /// Returns the number of elements in the slice.
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    #[must_use]
    /// Returns a pointer to the underlying slice, which may be null if the slice is empty.
    pub const fn as_ptr(&self) -> *const () {
        self.data
    }

    #[inline]
    #[must_use]
    /// Returns `true` if the slice has a length of 0.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    #[must_use]
    /// Returns a reference to the first element, without doing bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `!self.is_empty()`
    /// Calling this on an empty `DynSlice` will result in a segfault!
    pub unsafe fn first_unchecked(&self) -> &Dyn {
        debug_assert!(!self.is_empty(), "[dyn-slice] slice is empty!");
        debug_assert!(
            !self.vtable_ptr.is_null(),
            "[dyn-slice] vtable pointer is null on access!"
        );

        &*ptr::from_raw_parts::<Dyn>(self.as_ptr(), transmute(self.vtable_ptr()))
    }

    #[must_use]
    /// Returns a reference to the first element of the slice, or `None` if it is empty.
    pub fn first(&self) -> Option<&Dyn> {
        (!self.is_empty()).then(|| {
            // SAFETY:
            // The above statement ensures that slice is not empty, and
            // therefore has a first (index 0) element and a valid vtable pointer.
            unsafe { self.first_unchecked() }
        })
    }

    #[must_use]
    /// Returns a reference to the last element of the slice, or `None` if it is empty.
    pub fn last(&self) -> Option<&Dyn> {
        (!self.is_empty()).then(|| {
            // SAFETY:
            // The above statement ensures that slice is not empty, and
            // therefore has a last (index len - 1) element and a valid vtable pointer.
            unsafe { self.get_unchecked(self.len - 1) }
        })
    }

    #[must_use]
    /// Returns a reference to the element at the given `index` or `None` if the `index` is out of bounds.
    pub fn get(&self, index: usize) -> Option<&Dyn> {
        (index < self.len).then(|| {
            // SAFETY:
            // The above inequality ensures that the index is less than the
            // length, and is therefore valid. This also ensures that the slice
            // has a valid vtable pointer because the slice guaranteed to not be empty.
            unsafe { self.get_unchecked(index) }
        })
    }

    #[inline]
    #[must_use]
    /// Returns a reference to the element at the given `index`, without doing bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `index < self.len()`
    /// Calling this on an empty dyn Slice will result in a segfault!
    pub unsafe fn get_unchecked(&self, index: usize) -> &Dyn {
        debug_assert!(
            index < self.len,
            "[dyn-slice] index is greater than length!"
        );
        debug_assert!(
            !self.vtable_ptr.is_null(),
            "[dyn-slice] vtable pointer is null on access!"
        );

        let metadata = transmute::<_, DynMetadata<Dyn>>(self.vtable_ptr());
        &*ptr::from_raw_parts::<Dyn>(self.as_ptr().byte_add(metadata.size_of() * index), metadata)
    }

    #[inline]
    #[must_use]
    /// Get a sub-slice from the `start` index with the `len`, without doing bounds checking.
    ///
    /// # Safety
    /// Caller must ensure that:
    /// - `start` < `self.len()`
    /// - `len` <= `self.len() - start`
    pub unsafe fn slice_unchecked(&self, start: usize, len: usize) -> DynSlice<Dyn> {
        // NOTE: DO NOT MAKE THIS FUNCTION RETURN `Self` as `Self` comes with an incorrect lifetime
        debug_assert!(
            start + len <= self.len,
            "[dyn-slice] sub-slice is out of bounds!"
        );

        let metadata = transmute::<_, DynMetadata<Dyn>>(self.vtable_ptr());
        Self::from_parts_with_metadata(
            metadata,
            len,
            self.as_ptr().byte_add(metadata.size_of() * start),
        )
    }

    #[must_use]
    /// Returns a sub-slice from the `start` index with the `len` or `None` if the slice is out of bounds.
    pub fn slice<R: RangeBounds<usize>>(&self, range: R) -> Option<DynSlice<Dyn>> {
        // NOTE: DO NOT MAKE THIS FUNCTION RETURN `Self` as `Self` comes with an incorrect lifetime

        let start_inclusive = match range.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => i.checked_add(1)?,
            Bound::Unbounded => 0,
        };

        let end_exclusive = match range.end_bound() {
            Bound::Included(i) => i.checked_add(1)?,
            Bound::Excluded(i) => *i,
            Bound::Unbounded => self.len,
        };

        if end_exclusive > self.len {
            return None;
        }

        let len = end_exclusive.checked_sub(start_inclusive)?;

        // SAFETY:
        // The above `if` statement ensures that the the end of the new slice
        // does not exceed that of the original slice, therefore, the new
        // slice is valid.
        Some(unsafe { self.slice_unchecked(start_inclusive, len) })
    }

    #[inline]
    #[must_use]
    /// Returns the underlying slice as `&[T]`.
    ///
    /// # Safety
    /// The caller must ensure that the underlying slice is of type `[T]`.
    pub const unsafe fn downcast_unchecked<T>(&self) -> &[T] {
        slice::from_raw_parts(self.as_ptr().cast(), self.len)
    }

    #[inline]
    #[must_use]
    /// Returns an iterator over the slice.
    pub const fn iter(&'a self) -> Iter<'a, Dyn> {
        Iter { slice: *self }
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> Index<usize> for DynSlice<'a, Dyn> {
    type Output = Dyn;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "index out of bounds");
        debug_assert!(
            !self.vtable_ptr.is_null(),
            "[dyn-slice] vtable pointer is null on access!"
        );

        // SAFETY:
        // The above assertion ensures that the index is less than the
        // length, and is therefore valid. This also ensures that the slice
        // has a valid vtable pointer because the slice guaranteed to not be empty.
        unsafe { self.get_unchecked(index) }
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> IntoIterator for DynSlice<'a, Dyn> {
    type IntoIter = Iter<'a, Dyn>;
    type Item = &'a Dyn;

    fn into_iter(self) -> Self::IntoIter {
        Iter { slice: self }
    }
}

#[cfg(test)]
mod test {
    use core::{fmt::Display, ptr::addr_of};

    use crate::{declare_new_fn, standard::partial_eq, DynSlice};

    declare_new_fn!(Display, display_dyn_slice);
    pub use display_dyn_slice::new as new_display_dyn_slice;

    #[test]
    fn create_dyn_slice() {
        let array: [u8; 5] = [1, 2, 3, 4, 5];

        let dyn_slice = new_display_dyn_slice(&array);

        assert_eq!(dyn_slice.len(), array.len());
        assert!(!dyn_slice.is_empty());

        for (i, x) in array.iter().enumerate() {
            assert_eq!(
                format!(
                    "{}",
                    dyn_slice
                        .get(i)
                        .expect("failed to get an element of dyn_slice")
                ),
                format!("{x}"),
            );
        }
    }

    #[test]
    fn empty() {
        let array: [u8; 0] = [];

        let dyn_slice = new_display_dyn_slice(&array);

        assert_eq!(dyn_slice.len(), 0);
        assert!(dyn_slice.is_empty());
    }

    #[test]
    fn test_slice() {
        let array = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let slice = partial_eq::new(&array);
        assert_eq!(slice.len(), array.len());

        // Slices equal to the original slice
        let full_slices: [DynSlice<dyn PartialEq<i32>>; 6] = [
            slice.slice(..).unwrap(),
            slice.slice(0..).unwrap(),
            slice.slice(..(array.len())).unwrap(),
            #[allow(clippy::range_minus_one)]
            slice.slice(..=(array.len() - 1)).unwrap(),
            slice.slice(0..array.len()).unwrap(),
            #[allow(clippy::range_minus_one)]
            slice.slice(0..=(array.len() - 1)).unwrap(),
        ];

        for sub_slice in full_slices {
            assert_eq!(sub_slice.metadata(), slice.metadata());
            assert_eq!(sub_slice.len(), slice.len());
            assert_eq!(sub_slice.as_ptr(), slice.as_ptr());
        }

        // Sub-slices bounded on one side
        let sub_slice = slice.slice(2..).unwrap();
        assert_eq!(sub_slice.metadata(), slice.metadata());
        assert_eq!(sub_slice.len(), slice.len() - 2);
        assert_eq!(sub_slice.as_ptr(), addr_of!(slice[2]).cast());

        let sub_slice = slice.slice(..7).unwrap();
        assert_eq!(sub_slice.metadata(), slice.metadata());
        assert_eq!(sub_slice.len(), 7);
        assert_eq!(sub_slice.as_ptr(), slice.as_ptr());

        // Sub-slices bounded on both sides
        let sub_slices = [
            slice.slice(2..(array.len())).unwrap(),
            #[allow(clippy::range_minus_one)]
            slice.slice(2..=(array.len() - 1)).unwrap(),
        ];

        for sub_slice in sub_slices {
            assert_eq!(sub_slice.metadata(), slice.metadata());
            assert_eq!(sub_slice.len(), slice.len() - 2);
            assert_eq!(sub_slice.as_ptr(), addr_of!(slice[2]).cast());
        }

        // Sub-slices with zero length
        let zero_length_slices = [
            slice.slice(0..0).unwrap(),
            slice.slice(2..2).unwrap(),
            #[allow(clippy::reversed_empty_ranges)]
            slice.slice(2..=1).unwrap(),
            slice.slice((array.len())..).unwrap(),
        ];

        for sub_slice in zero_length_slices {
            assert_eq!(sub_slice.metadata(), slice.metadata());
            assert_eq!(sub_slice.len(), 0);
        }

        // Invalid sub-slices
        let invalid_slices = [
            #[allow(clippy::range_plus_one)]
            slice.slice(..(array.len() + 1)),
            slice.slice(..=(array.len())),
        ];

        for sub_slice in invalid_slices {
            assert!(sub_slice.is_none());
        }
    }
}
