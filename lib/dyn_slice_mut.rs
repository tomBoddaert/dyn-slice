use core::{
    mem::transmute,
    ops::{Bound, Deref, Index, IndexMut, RangeBounds},
    ptr::{self, DynMetadata, Pointee},
    slice,
};

use crate::{DynSlice, Iter, IterMut};

/// `&mut dyn [Trait]`
///
/// A mutable type erased slice of elements that implement a trait.
///
/// # Example
/// ```
/// use dyn_slice::standard::add_assign;
///
/// let mut array = [1, 2, 3, 4, 5];
/// let mut slice = add_assign::new_mut(&mut array);
/// slice.iter_mut().for_each(|x| *x += 10);
/// assert_eq!(array, [11, 12, 13, 14, 15]);
/// ```
#[repr(transparent)]
pub struct DynSliceMut<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>>(
    pub(crate) DynSlice<'a, Dyn>,
);

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> AsRef<DynSlice<'a, Dyn>>
    for DynSliceMut<'a, Dyn>
{
    #[inline]
    fn as_ref(&self) -> &DynSlice<'a, Dyn> {
        &self.0
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> Deref for DynSliceMut<'a, Dyn> {
    type Target = DynSlice<'a, Dyn>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> DynSliceMut<'a, Dyn> {
    #[inline]
    #[must_use]
    /// Construct a mutable dyn slice given a mutable slice and a vtable pointer.
    ///
    /// # Safety
    /// Caller must ensure that `vtable_ptr` is a valid instance of `DynMetadata` for `DynSliceFromType` and `Dyn` transmuted, or optionally, a null pointer if `value.len() == 0`.
    pub unsafe fn with_vtable_ptr<DynSliceFromType>(
        value: &'a mut [DynSliceFromType],
        vtable_ptr: *const (),
    ) -> Self {
        Self(DynSlice::with_vtable_ptr(value, vtable_ptr))
    }

    #[inline]
    #[must_use]
    /// Construct a mutable dyn slice given a mutable slice and a `DynMetadata` instance.
    ///
    /// # Safety
    /// Caller must ensure that `metadata` is a valid instance of `DynMetadata` for `DynSliceFromType` and `Dyn`.
    pub unsafe fn with_metadata<DynSliceFromType>(
        value: &'a mut [DynSliceFromType],
        metadata: DynMetadata<Dyn>,
    ) -> Self {
        Self::with_vtable_ptr(value, transmute(metadata))
    }

    #[inline]
    #[must_use]
    /// Construct a mutable dyn slice from raw parts.
    ///
    /// # Safety
    /// Caller must ensure that:
    /// - `vtable_ptr` is a valid instance of `DynMetadata` transmuted, or optionally, a null pointer if `len == 0`,
    /// - `len` <= the length of the slice in memory from the `data` pointer,
    /// - `data` is a valid pointer to the slice,
    /// - the underlying slice is the same layout as [`[T]`](https://doc.rust-lang.org/reference/type-layout.html#slice-layout)
    pub const unsafe fn from_parts(vtable_ptr: *const (), len: usize, data: *mut ()) -> Self {
        Self(DynSlice::from_parts(vtable_ptr, len, data))
    }

    #[inline]
    #[must_use]
    /// Construct a mutable dyn slice from raw parts with a `DynMetadata` instance rather than a vtable pointer.
    ///
    /// # Safety
    /// Caller must ensure that:
    /// - `metadata` is a valid instance of `DynMetadata`,
    /// - `len` <= the length of the slice in memory from the `data` pointer,
    /// - `data` is a valid pointer to the slice,
    /// - the underlying slice is the same layout as [`[T]`](https://doc.rust-lang.org/reference/type-layout.html#slice-layout)
    pub const unsafe fn from_parts_with_metadata(
        metadata: DynMetadata<Dyn>,
        len: usize,
        data: *mut (),
    ) -> Self {
        Self::from_parts(transmute(metadata), len, data)
    }

    #[inline]
    #[must_use]
    /// Returns a mutable pointer to the underlying slice, which may be null if the slice is empty.
    pub fn as_mut_ptr(&mut self) -> *mut () {
        self.0.data.cast_mut()
    }

    #[inline]
    #[must_use]
    /// Returns a mutable reference to the first element, without doing bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `!self.is_empty()`
    /// Calling this on an empty `DynSlice` will result in a segfault!
    pub unsafe fn first_unchecked_mut(&mut self) -> &mut Dyn {
        debug_assert!(!self.is_empty(), "[dyn-slice] slice is empty!");
        debug_assert!(
            !self.vtable_ptr.is_null(),
            "[dyn-slice] vtable pointer is null on access!"
        );

        &mut *ptr::from_raw_parts_mut::<Dyn>(self.as_mut_ptr(), transmute(self.vtable_ptr()))
    }

    #[must_use]
    /// Returns a mutable reference to the first element of the slice, or `None` if it is empty.
    ///
    /// # Example
    /// ```
    /// use dyn_slice::standard::add_assign;
    ///
    /// let mut array = [1, 2, 3, 4, 5];
    /// let mut slice = add_assign::new_mut(&mut array);
    ///
    /// *slice.first_mut().unwrap() += 10;
    /// assert_eq!(array, [11, 2, 3, 4, 5]);
    /// ```
    pub fn first_mut(&mut self) -> Option<&mut Dyn> {
        (!self.0.is_empty()).then(|| {
            debug_assert!(
                !self.vtable_ptr.is_null(),
                "[dyn-slice] vtable pointer is null on access!"
            );

            // SAFETY:
            // The above statement ensures that slice is not empty, and
            // therefore has a first (index 0) element and a valid vtable pointer.
            unsafe { self.first_unchecked_mut() }
        })
    }

    #[must_use]
    /// Returns a mutable reference to the last element of the slice, or `None` if it is empty.
    ///
    /// # Example
    /// ```
    /// use dyn_slice::standard::add_assign;
    ///
    /// let mut array = [1, 2, 3, 4, 5];
    /// let mut slice = add_assign::new_mut(&mut array);
    ///
    /// *slice.last_mut().unwrap() += 10;
    /// assert_eq!(array, [1, 2, 3, 4, 15]);
    /// ```
    pub fn last_mut(&mut self) -> Option<&mut Dyn> {
        (!self.0.is_empty()).then(|| {
            // SAFETY:
            // The above statement ensures that slice is not empty, and
            // therefore has a last (index len - 1) element and a valid vtable pointer.
            unsafe { self.get_unchecked_mut(self.0.len - 1) }
        })
    }

    #[must_use]
    /// Returns a mutable reference to the element at the given `index` or `None` if the `index` is out of bounds.
    ///
    /// # Example
    /// ```
    /// use dyn_slice::standard::add_assign;
    ///
    /// let mut array = [1, 2, 3, 4, 5];
    /// let mut slice = add_assign::new_mut(&mut array);
    ///
    /// *slice.get_mut(2).unwrap() += 10;
    /// assert_eq!(array, [1, 2, 13, 4, 5]);
    /// ```
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Dyn> {
        (index < self.0.len).then(|| {
            // SAFETY:
            // The above inequality ensures that the index is less than the
            // length, and is therefore valid. This also ensures that the slice
            // has a valid vtable pointer because the slice guaranteed to not be empty.
            unsafe { self.get_unchecked_mut(index) }
        })
    }

    #[inline]
    #[must_use]
    /// Returns a mutable reference to the element at the given `index`, without doing bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that `index < self.len()`
    /// Calling this on an empty dyn Slice will result in a segfault!
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Dyn {
        debug_assert!(
            index < self.len,
            "[dyn-slice] index is greater than length!"
        );
        debug_assert!(
            !self.vtable_ptr.is_null(),
            "[dyn-slice] vtable pointer is null on access!"
        );

        let metadata = transmute::<_, DynMetadata<Dyn>>(self.0.vtable_ptr());
        &mut *ptr::from_raw_parts_mut::<Dyn>(
            self.as_mut_ptr().byte_add(metadata.size_of() * index),
            metadata,
        )
    }

    #[inline]
    #[must_use]
    /// Get a mutable sub-slice from the `start` index with the `len`, without doing bounds checking.
    ///
    /// # Safety
    /// Caller must ensure that:
    /// - `start < self.len()`
    /// - `len <= self.len() - start`
    pub unsafe fn slice_unchecked_mut(&mut self, start: usize, len: usize) -> DynSliceMut<Dyn> {
        // NOTE: DO NOT MAKE THIS FUNCTION RETURN `Self` as `Self` comes with an incorrect lifetime
        debug_assert!(
            start + len <= self.len,
            "[dyn-slice] sub-slice is out of bounds!"
        );

        let metadata = transmute::<_, DynMetadata<Dyn>>(self.0.vtable_ptr());
        Self::from_parts_with_metadata(
            metadata,
            len,
            self.as_mut_ptr().byte_add(metadata.size_of() * start),
        )
    }

    #[must_use]
    /// Returns a mutable sub-slice from the `start` index with the `len` or `None` if the slice is out of bounds.
    ///
    /// # Example
    /// ```
    /// use dyn_slice::standard::add_assign;
    ///
    /// let mut array = [1, 2, 3, 4, 5];
    /// let mut slice = add_assign::new_mut(&mut array);
    ///
    /// slice.slice_mut(1..4).unwrap().iter_mut().for_each(|x| *x += 10);
    /// slice.slice_mut(2..).unwrap().iter_mut().for_each(|x| *x += 10);
    /// slice.slice_mut(5..).unwrap().iter_mut().for_each(|x| *x += 10);
    /// assert!(slice.slice(6..).is_none());
    /// assert_eq!(array, [1, 12, 23, 24, 15]);
    /// ```
    pub fn slice_mut<R: RangeBounds<usize>>(&mut self, range: R) -> Option<DynSliceMut<Dyn>> {
        // NOTE: DO NOT MAKE THIS FUNCTION RETURN `Self` as `Self` comes with an incorrect lifetime

        let start_inclusive = match range.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => i.checked_add(1)?,
            Bound::Unbounded => 0,
        };

        let end_exclusive = match range.end_bound() {
            Bound::Included(i) => i.checked_add(1)?,
            Bound::Excluded(i) => *i,
            Bound::Unbounded => self.0.len,
        };

        if end_exclusive > self.0.len {
            return None;
        }

        let len = end_exclusive.checked_sub(start_inclusive)?;

        // SAFETY:
        // The above `if` statement ensures that the the end of the new slice
        // does not exceed that of the original slice, therefore, the new
        // slice is valid.
        Some(unsafe { self.slice_unchecked_mut(start_inclusive, len) })
    }

    #[inline]
    #[must_use]
    /// Returns the underlying slice as `&mut [T]`.
    ///
    /// # Safety
    /// The caller must ensure that the underlying slice is of type `[T]`.
    pub unsafe fn downcast_unchecked_mut<T>(&mut self) -> &mut [T] {
        slice::from_raw_parts_mut(self.as_ptr().cast_mut().cast(), self.len)
    }

    #[inline]
    #[must_use]
    /// Returns a mutable iterator over the slice.
    ///
    /// # Example
    /// ```
    /// use dyn_slice::standard::add_assign;
    ///
    /// let mut array = [1, 2, 3, 4, 5];
    /// let mut slice = add_assign::new_mut(&mut array);
    ///
    /// slice.iter_mut().for_each(|x| *x += 10);
    /// assert_eq!(array, [11, 12, 13, 14, 15]);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, Dyn> {
        IterMut {
            // SAFETY:
            // The created slice is from index 0 and has the same length as the
            // original slice, so must be valid.
            slice: unsafe { self.slice_unchecked_mut(0, self.len) },
        }
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> Index<usize> for DynSliceMut<'a, Dyn> {
    type Output = Dyn;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> IndexMut<usize>
    for DynSliceMut<'a, Dyn>
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.0.len, "index out of bounds");
        debug_assert!(
            !self.vtable_ptr.is_null(),
            "[dyn-slice] vtable pointer is null on access!"
        );

        // SAFETY:
        // The above assertion ensures that the index is less than the
        // length, and is therefore valid. This also ensures that the slice
        // has a valid vtable pointer because the slice guaranteed to not be empty.
        unsafe { self.get_unchecked_mut(index) }
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> IntoIterator for DynSliceMut<'a, Dyn> {
    type IntoIter = IterMut<'a, Dyn>;
    type Item = &'a mut Dyn;

    fn into_iter(self) -> Self::IntoIter {
        IterMut { slice: self }
    }
}

impl<'a, 'b, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> IntoIterator
    for &'b mut DynSliceMut<'a, Dyn>
{
    type IntoIter = IterMut<'b, Dyn>;
    type Item = &'b mut Dyn;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, 'b, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> IntoIterator
    for &'b DynSliceMut<'a, Dyn>
{
    type IntoIter = Iter<'b, Dyn>;
    type Item = &'b Dyn;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod test {
    use core::{fmt::Display, ptr::addr_of};

    use crate::{declare_new_fns, standard::partial_eq, DynSliceMut};

    declare_new_fns!(
        #[crate = crate]
        display_dyn_slice Display
    );
    pub use display_dyn_slice::new_mut as new_display_dyn_slice;

    #[test]
    fn create_dyn_slice() {
        let array: [u8; 5] = [1, 2, 3, 4, 5];
        let mut array2 = array;

        let dyn_slice = new_display_dyn_slice(&mut array2);

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
        let mut array: [u8; 0] = [];

        let dyn_slice = new_display_dyn_slice(&mut array);

        assert_eq!(dyn_slice.len(), 0);
        assert!(dyn_slice.is_empty());
    }

    #[test]
    fn test_slice() {
        type GetSliceFn = for<'a> fn(
            &'a mut DynSliceMut<dyn PartialEq<i32>>,
        ) -> DynSliceMut<'a, dyn PartialEq<i32>>;
        type GetOptSliceFn = for<'a> fn(
            &'a mut DynSliceMut<dyn PartialEq<i32>>,
        ) -> Option<DynSliceMut<'a, dyn PartialEq<i32>>>;

        let mut array = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let len = array.len();
        let mut slice = partial_eq::new_mut(&mut array);
        let metadata = slice.metadata().unwrap();
        assert_eq!(slice.len(), len);

        // Slices equal to the original slice
        let full_slices: [GetSliceFn; 6] = [
            |slice| slice.slice_mut(..).unwrap(),
            |slice| slice.slice_mut(0..).unwrap(),
            |slice| slice.slice_mut(..(slice.len())).unwrap(),
            #[allow(clippy::range_minus_one)]
            |slice| slice.slice_mut(..=(slice.len() - 1)).unwrap(),
            |slice| slice.slice_mut(0..slice.len()).unwrap(),
            #[allow(clippy::range_minus_one)]
            |slice| slice.slice_mut(0..=(slice.len() - 1)).unwrap(),
        ];

        for get_sub_slice in full_slices {
            let sub_slice = get_sub_slice(&mut slice);

            assert_eq!(sub_slice.metadata(), Some(metadata));
            assert_eq!(sub_slice.len(), len);
            assert_eq!(sub_slice.as_ptr(), slice.as_ptr());
        }

        // Sub-slices bounded on one side
        let sub_slice = slice.slice_mut(2..).unwrap();
        assert_eq!(sub_slice.metadata(), Some(metadata));
        assert_eq!(sub_slice.len(), len - 2);
        assert_eq!(sub_slice.as_ptr(), addr_of!(slice[2]).cast());

        let sub_slice = slice.slice_mut(..7).unwrap();
        assert_eq!(sub_slice.metadata(), Some(metadata));
        assert_eq!(sub_slice.len(), 7);
        assert_eq!(sub_slice.as_ptr(), slice.as_ptr());

        // Sub-slices bounded on both sides
        let sub_slices: [GetSliceFn; 2] = [
            |slice| slice.slice_mut(2..slice.len()).unwrap(),
            #[allow(clippy::range_minus_one)]
            |slice| slice.slice_mut(2..=(slice.len() - 1)).unwrap(),
        ];

        for get_sub_slice in sub_slices {
            let sub_slice = get_sub_slice(&mut slice);

            assert_eq!(sub_slice.metadata(), Some(metadata));
            assert_eq!(sub_slice.len(), len - 2);
            assert_eq!(sub_slice.as_ptr(), addr_of!(slice[2]).cast());
        }

        // Sub-slices with zero length
        let zero_length_slices: [GetSliceFn; 4] = [
            |slice| slice.slice_mut(0..0).unwrap(),
            |slice| slice.slice_mut(2..2).unwrap(),
            #[allow(clippy::reversed_empty_ranges)]
            |slice| slice.slice_mut(2..=1).unwrap(),
            |slice| slice.slice_mut((slice.len())..).unwrap(),
        ];

        for get_sub_slice in zero_length_slices {
            let sub_slice = get_sub_slice(&mut slice);

            assert_eq!(sub_slice.metadata(), Some(metadata));
            assert_eq!(sub_slice.len(), 0);
        }

        // Invalid sub-slices
        let invalid_slices: [GetOptSliceFn; 2] = [
            #[allow(clippy::range_plus_one)]
            |slice| slice.slice_mut(..(slice.len() + 1)),
            |slice| slice.slice_mut(..=(slice.len())),
        ];

        for get_sub_slice in invalid_slices {
            let sub_slice = get_sub_slice(&mut slice);

            assert!(sub_slice.is_none());
        }
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn index_empty() {
        let slice = partial_eq::new_mut::<u8, u8>(&mut []);
        _ = &slice[0];
    }

    #[test]
    fn index() {
        let mut array = [1, 2, 3, 4];
        let slice = partial_eq::new_mut::<u8, u8>(&mut array);
        assert!(slice[0] == 1);
        assert!(slice[1] == 2);
        assert!(slice[2] == 3);
        assert!(slice[3] == 4);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn index_on_bound() {
        let mut array = [1, 2, 3, 4];
        let slice = partial_eq::new_mut::<u8, u8>(&mut array);
        _ = &slice[4];
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn index_out_of_bounds() {
        let mut array = [1, 2, 3, 4];
        let slice = partial_eq::new_mut::<u8, u8>(&mut array);
        _ = &slice[6];
    }
}
