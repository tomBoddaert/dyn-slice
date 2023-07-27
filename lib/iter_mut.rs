use core::{
    iter::FusedIterator,
    mem::transmute,
    ptr::{DynMetadata, Pointee},
};

use crate::DynSliceMut;

/// Mutable dyn slice iterator
pub struct IterMut<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> {
    pub(crate) slice: DynSliceMut<'a, Dyn>,
    pub(crate) next_index: usize,
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> Iterator for IterMut<'a, Dyn> {
    type Item = &'a mut Dyn;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index == self.slice.len() {
            None
        } else {
            // SAFETY:
            // As the index is incremented by this method only, and it is checked to make sure
            // it is not equal to the length, the index is guaranteed to be less than the length
            // and therefore, valid. This also ensures that the slice
            // has a valid vtable pointer because the slice guaranteed to not be empty.
            let element = unsafe { self.slice.get_unchecked_mut(self.next_index) };
            self.next_index += 1;

            // SAFETY:
            // The data is guaranteed to live for at least 'a, and not have another reference to it
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

    fn last(mut self) -> Option<Self::Item> {
        if self.next_index == self.slice.len() {
            None
        } else {
            // SAFETY:
            // The data is guaranteed to live for at least 'a, and not have another reference to it
            // in that time, so the lifetime can be extended.
            unsafe { transmute(self.slice.last_mut()) }
        }
    }
}

impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + 'a> FusedIterator for IterMut<'a, Dyn> {}
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + 'a> ExactSizeIterator for IterMut<'a, Dyn> {}
