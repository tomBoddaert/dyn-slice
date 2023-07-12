use core::{
    ops::RangeBounds,
    ptr::{DynMetadata, Pointee},
};

/// The trait for dyn slices.
///
/// This trait implements most methods for dyn slices, as defined by the [`super::declare_dyn_slice`] macro.
///
/// **You should not need to implement this manually!**  
/// It is implemented by the [`super::declare_dyn_slice`] macro.
///
/// # Safety
/// Implementer must ensure that:
/// - `metadata` yields a valid instance of `DynMetadata` for the type contained in the slice,
//  - `len` yields the correct length of the underlying slice,
/// - `as_ptr` yields the pointer to the start of the underlying slice,
/// - the underlying slice has the same layout as [`[T]`](https://doc.rust-lang.org/reference/type-layout.html#slice-layout),
/// - the implementing type must not live for longer than the underlying slice
pub unsafe trait DynSliceTrait: Sized {
    /// The unsized, dynamic type (`dyn $Trait`)
    type Dyn: ?Sized + Pointee<Metadata = DynMetadata<Self::Dyn>>;

    /// # Safety
    /// Caller must ensure that:
    /// - `metadata` is a valid instance of `DynMetadata`,
    /// - `len` <= then length of the slice in memory from the `data` pointer,
    /// - `data` is a valid pointer to the slice,
    /// - the underlying slice is the same layout as [`[T]`](https://doc.rust-lang.org/reference/type-layout.html#slice-layout)
    unsafe fn from_parts(metadata: DynMetadata<Self::Dyn>, len: usize, data: *const ()) -> Self;

    /// Get the metadata component of the element's pointers.
    fn metadata(&self) -> DynMetadata<Self::Dyn>;
    /// Returns the number of elements in the slice.
    fn len(&self) -> usize;
    /// Returns a pointer to the underlying slice.
    fn as_ptr(&self) -> *const ();

    #[inline]
    #[must_use]
    /// Returns `true` if the slice has a length of 0.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    /// Returns a reference to the first element of the slice, or `None` if it is empty.
    fn first(&self) -> Option<&Self::Dyn> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe {
                core::ptr::from_raw_parts::<Self::Dyn>(self.as_ptr(), self.metadata())
                    .as_ref()
                    .unwrap()
            })
        }
    }

    #[must_use]
    /// Returns a reference to the last element of the slice, or `None` if it is empty.
    fn last(&self) -> Option<&Self::Dyn> {
        if self.is_empty() {
            None
        } else {
            Some(unsafe { self.get_unchecked(self.len() - 1) })
        }
    }

    #[must_use]
    /// Returns a reference to the element at the given `index` or `None` if the `index` is out of bounds.
    fn get(&self, index: usize) -> Option<&Self::Dyn> {
        if index >= self.len() {
            None
        } else {
            Some(unsafe { self.get_unchecked(index) })
        }
    }

    #[must_use]
    /// Returns a reference to the element at the given `index`, without doing bounds checking.
    ///
    /// # Safety
    /// The caller must ensure that index < self.len()
    /// Calling this on an empty dyn Slice will result in a segfault!
    unsafe fn get_unchecked(&self, index: usize) -> &Self::Dyn {
        core::ptr::from_raw_parts::<Self::Dyn>(
            self.as_ptr().byte_add(self.metadata().size_of() * index),
            self.metadata(),
        )
        .as_ref()
        .unwrap()
    }

    #[must_use]
    /// # Safety
    /// Caller must ensure that:
    /// - `start` < `self.len()`
    /// - `len` <= `self.len() - start`
    unsafe fn slice_unchecked(&self, start: usize, len: usize) -> Self {
        let metadata = self.metadata();
        Self::from_parts(
            metadata,
            len,
            self.as_ptr().byte_add(metadata.size_of() * start),
        )
    }

    #[must_use]
    fn slice<R: RangeBounds<usize>>(&self, range: R) -> Option<Self> {
        use core::ops::Bound;

        let start_inclusive = match range.start_bound() {
            Bound::Included(i) => *i,
            Bound::Excluded(i) => i.checked_add(1)?,
            Bound::Unbounded => 0,
        };

        let end_exclusive = match range.end_bound() {
            Bound::Included(i) => i.checked_add(1)?,
            Bound::Excluded(i) => *i,
            Bound::Unbounded => self.len(),
        };

        if end_exclusive > self.len() {
            return None;
        }

        let len = end_exclusive.checked_sub(start_inclusive)?;

        Some(unsafe { self.slice_unchecked(start_inclusive, len) })
    }

    #[inline]
    #[must_use]
    /// Returns an iterator over the slice.
    fn iter(&self) -> Iter<Self> {
        Iter {
            slice: self,
            next_index: 0,
        }
    }
}

#[derive(Clone)]
/// Dyn slice iterator
pub struct Iter<'a, DS: DynSliceTrait + 'a> {
    slice: &'a DS,
    next_index: usize,
}

impl<'a, DS: DynSliceTrait + 'a> Iterator for Iter<'a, DS> {
    type Item = &'a DS::Dyn;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index == self.slice.len() {
            None
        } else {
            let element = unsafe { self.slice.get_unchecked(self.next_index) };
            self.next_index += 1;

            Some(element)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.slice.len() - self.next_index;
        (remaining, Some(remaining))
    }

    #[inline]
    fn count(self) -> usize {
        self.slice.len() - self.next_index
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = self.next_index + n;
        if index >= self.slice.len() {
            self.next_index = self.slice.len();
            return None;
        }

        self.next_index = index;
        self.next()
    }

    fn last(self) -> Option<Self::Item> {
        if self.next_index == self.slice.len() {
            None
        } else {
            self.slice.last()
        }
    }
}

impl<'a, DS: DynSliceTrait + 'a> core::iter::FusedIterator for Iter<'a, DS> {}
impl<'a, DS: DynSliceTrait + 'a> ExactSizeIterator for Iter<'a, DS> {}

#[cfg(test)]
mod test {
    use core::ptr::addr_of;

    use crate::standard::PartialEqDynSlice;

    use super::DynSliceTrait;

    #[test]
    fn test_slice() {
        let array = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        let slice = PartialEqDynSlice::new(&array);
        assert_eq!(slice.len(), array.len());

        // Slices equal to the original slice
        let full_slices = [
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