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
            let element = unsafe { self.slice.get_unchecked(self.next_index) };
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

    fn last(self) -> Option<Self::Item> {
        if self.next_index == self.slice.len() {
            None
        } else {
            // The lifetime can be extended because the data will not
            // be accessed again until the iterator's lifetime expires
            unsafe { transmute(self.slice.last()) }
        }
    }
}

impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + 'a> FusedIterator for Iter<'a, Dyn> {}
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + 'a> ExactSizeIterator for Iter<'a, Dyn> {}
