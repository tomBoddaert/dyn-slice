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
            let element = unsafe { self.slice.get_unchecked_mut(self.next_index) };
            self.next_index += 1;

            // The lifetime can be extended because the data will not
            // be accessed again until the iterator's lifetime expires
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
            // The lifetime can be extended because the data will not
            // be accessed again until the iterator's lifetime expires
            unsafe { transmute(self.slice.last_mut()) }
        }
    }
}

impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + 'a> FusedIterator for IterMut<'a, Dyn> {}
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + 'a> ExactSizeIterator for IterMut<'a, Dyn> {}
