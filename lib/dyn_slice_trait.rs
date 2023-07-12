use core::ptr::{DynMetadata, Pointee};

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
        unsafe {
            core::ptr::from_raw_parts::<Self::Dyn>(
                self.as_ptr().byte_add(self.metadata().size_of() * index),
                self.metadata(),
            )
            .as_ref()
            .unwrap()
        }
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
