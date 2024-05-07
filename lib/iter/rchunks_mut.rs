use core::{
    cmp,
    num::NonZeroUsize,
    ptr::{DynMetadata, Pointee},
};

use crate::{utils::extend_lifetime_mut, DynSliceMut};

/// Iterator over non-overlapping chunks of a [`DynSliceMut`] from right to left.
pub struct RChunksMut<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> {
    pub(crate) slice: DynSliceMut<'a, Dyn>,
    pub(crate) chunk_size: NonZeroUsize,
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> Iterator for RChunksMut<'a, Dyn> {
    type Item = DynSliceMut<'a, Dyn>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let len = cmp::min(self.slice.len(), self.chunk_size.get());

            // len <= slice length, so this cannot underflow
            let mid = self.slice.len() - len;

            // SAFETY:
            // `len` is upper bounded by the slice length, so splitting
            // here is valid.
            let (remaining, chunk) = unsafe { self.slice.split_at_unchecked_mut(mid) };
            let (remaining, chunk) =
                // SAFETY:
                // The original slice is immediately replaced with one part,
                // so the lifetimes can be extended to match it.
                unsafe { (extend_lifetime_mut(remaining), extend_lifetime_mut(chunk)) };
            self.slice = remaining;

            Some(chunk)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Use impl for ExactSizeIterator
        let remaining = self.len();
        (remaining, Some(remaining))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        // Get the number of elements that should be skipped
        let Some(skip_len) = self.chunk_size.get().checked_mul(n) else {
            self.slice.0.len = 0;
            return None;
        };

        self.slice.0.len = self.slice.len.saturating_sub(skip_len);

        self.next()
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> DoubleEndedIterator
    for RChunksMut<'a, Dyn>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            // Upper bounded by slice length
            let mut len = self.slice.len() % self.chunk_size;
            // Slice length != 0, so slice length >= chunk size
            if len == 0 {
                len = self.chunk_size.get();
            }

            // SAFETY:
            // `len` is upperbounded by the slice length.
            let (chunk, remaining) = unsafe { self.slice.split_at_unchecked_mut(len) };
            let (chunk, remaining) =
                // SAFETY:
                // The original slice is immediately replaced with one part,
                // so the lifetimes can be extended to match it.
                unsafe { (extend_lifetime_mut(chunk), extend_lifetime_mut(remaining)) };
            self.slice = remaining;

            Some(chunk)
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if self.slice.is_empty() {
            return None;
        }

        if let Some(m) = n.checked_sub(1) {
            // Get the length of all but the last chunk
            let Some(mut skipped) = m.checked_mul(self.chunk_size.get()) else {
                self.slice.0.len = 0;
                return None;
            };

            // Get the length of the last chunk
            let mut last = self.slice.len() % self.chunk_size;
            if last == 0 {
                // The slice is not empty as per the first condition
                last = self.chunk_size.get();
            }

            // Add the last chunk
            skipped = skipped.saturating_add(last);

            let Some(remaining) = self.slice.slice_mut(skipped..) else {
                self.slice.0.len = 0;
                return None;
            };
            // SAFETY:
            // The original slice is immediately replaced with the slice,
            // so the lifetime can be extended to match it.
            let remaining = unsafe { extend_lifetime_mut(remaining) };
            self.slice = remaining;
        }

        self.next_back()
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> ExactSizeIterator
    for RChunksMut<'a, Dyn>
{
    fn len(&self) -> usize {
        // Divide the length by the chunk size, then add one if the chunk size
        // does not exactly divide the length
        // This is done this way to avoid integer overflows for large chunk sizes
        self.slice.len() / self.chunk_size + usize::from(self.slice.len() % self.chunk_size != 0)
    }
}

#[cfg(test)]
mod test {
    use crate::test::{ped, test_iter};

    #[test]
    fn basic() {
        test_iter! {
            mut [1, 2, 3, 4, 5, 6],
            ds => ds.rchunks_mut(3).unwrap(),
            s => s.rchunks(3),
        }

        test_iter! {
            mut [1, 2, 3, 4, 5],
            ds => ds.rchunks_mut(3).unwrap(),
            s => s.rchunks(3),
        }
    }

    #[test]
    fn basic_back() {
        test_iter! {
            mut [1, 2, 3, 4, 5, 6],
            ds => ds.rchunks_mut(3).unwrap().rev(),
            s => s.rchunks(3).rev(),
        }

        test_iter! {
            mut [1, 2, 3, 4, 5],
            ds => ds.rchunks_mut(3).unwrap().rev(),
            s => s.rchunks(3).rev(),
        }
    }

    #[test]
    fn nth() {
        test_iter! {@nth
            mut [1, 2, 3, 4, 5, 6],
            ds => ds.rchunks_mut(3).unwrap(),
            s => s.rchunks(3),
        }

        test_iter! {@nth
            mut [1, 2, 3, 4, 5],
            ds => ds.rchunks_mut(3).unwrap(),
            s => s.rchunks(3),
        }
    }

    #[test]
    fn nth_back() {
        test_iter! {@nth
            mut [1, 2, 3, 4, 5, 6],
            ds => ds.rchunks_mut(3).unwrap().rev(),
            s => s.rchunks(3).rev(),
        }

        test_iter! {@nth
            mut [1, 2, 3, 4, 5],
            ds => ds.rchunks_mut(3).unwrap().rev(),
            s => s.rchunks(3).rev(),
        }
    }
}
