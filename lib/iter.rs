use core::{
    iter::FusedIterator,
    mem::transmute,
    ptr::{DynMetadata, Pointee},
};

use crate::DynSlice;

#[derive(Clone)]
/// Dyn slice iterator
pub struct Iter<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> {
    pub(crate) slice: DynSlice<'a, Dyn>,
    pub(crate) next_index: usize,
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> Iterator for Iter<'a, Dyn> {
    type Item = &'a Dyn;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index == self.slice.len() {
            None
        } else {
            // SAFETY:
            // As the index is incremented by this method only, and it is checked to make sure
            // it is not equal to the length, the index is guaranteed to be less than the length
            // and therefore, valid. This also ensures that the slice
            // has a valid vtable pointer because the slice guaranteed to not be empty.
            let element = unsafe { self.slice.get_unchecked(self.next_index) };
            self.next_index += 1;

            // SAFETY:
            // The data is guaranteed to live for at least 'a, and not have a mutable reference to it
            // in that time, so the lifetime can be extended.
            Some(unsafe { transmute(element) })
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
            // SAFETY:
            // The data is guaranteed to live for at least 'a, and not have a mutable reference to it
            // in that time, so the lifetime can be extended.
            unsafe { transmute(self.slice.last()) }
        }
    }
}

impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + 'a> FusedIterator for Iter<'a, Dyn> {}
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + 'a> ExactSizeIterator for Iter<'a, Dyn> {}
