use core::{
    iter::FusedIterator,
    num::NonZeroUsize,
    ptr::{DynMetadata, Pointee},
};

use crate::{utils::extend_lifetime, DynSlice};

/// Iterator over overlapping subslices of a [`DynSlice`].
pub struct Windows<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> {
    pub(crate) slice: DynSlice<'a, Dyn>,
    pub(crate) window_size: NonZeroUsize,
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> Iterator for Windows<'a, Dyn> {
    type Item = DynSlice<'a, Dyn>;

    fn next(&mut self) -> Option<Self::Item> {
        let window = self.slice.slice(..self.window_size.get())?;
        // SAFETY:
        // Given that this is an immutable slice, we can have multiple
        // references to it with the same lifetime.
        let window = unsafe { extend_lifetime(window) };

        // SAFETY:
        // Slicing from `window_size >= 1` above ensures that `length >= 1`,
        // therefore, slicing from `1..` is valid, and the new length will
        // be `length - 1`.
        let remaining = unsafe { self.slice.slice_unchecked(1, self.slice.len() - 1) };
        // SAFETY:
        // The original slice is immediately replaced with the new subslice.
        let remaining = unsafe { extend_lifetime(remaining) };
        self.slice = remaining;

        Some(window)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Use impl for ExactSizeIterator
        let len = self.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let remaining = self.slice.slice(n..)?;
        // SAFETY:
        // The original slice is immediately replaced with the new subslice.
        let remaining = unsafe { extend_lifetime(remaining) };
        self.slice = remaining;

        self.next()
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> DoubleEndedIterator
    for Windows<'a, Dyn>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mid = self.slice.len().checked_sub(self.window_size.get())?;
        // SAFETY:
        // As checked above, there are at least `window_size` elements
        // in the slice, so slicing at `mid = len - window_size` is valid.
        // The length is exactly the window size.
        let window = unsafe { self.slice.slice_unchecked(mid, self.window_size.get()) };
        // SAFETY:
        // Given that this is an immutable slice, we can have multiple
        // references to it with the same lifetime.
        let window = unsafe { extend_lifetime(window) };

        self.slice.len -= 1;

        Some(window)
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.slice.len = self.slice.len.saturating_sub(n);
        self.next_back()
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> ExactSizeIterator
    for Windows<'a, Dyn>
{
    #[inline]
    fn len(&self) -> usize {
        self.slice.len().saturating_sub(self.window_size.get() - 1)
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> FusedIterator
    for Windows<'a, Dyn>
{
}

#[cfg(test)]
mod test {
    use crate::test::{ped, test_iter};

    #[test]
    fn basic() {
        test_iter! {
            [1, 2, 3, 4, 5, 6],
            ds => ds.windows(3).unwrap(),
            s => s.windows(3),
        }

        test_iter! {
            [1, 2, 3, 4, 5],
            ds => ds.windows(3).unwrap(),
            s => s.windows(3),
        }
    }

    #[test]
    fn basic_back() {
        test_iter! {
            [1, 2, 3, 4, 5, 6],
            ds => ds.windows(3).unwrap().rev(),
            s => s.windows(3).rev(),
        }

        test_iter! {
            [1, 2, 3, 4, 5],
            ds => ds.windows(3).unwrap().rev(),
            s => s.windows(3).rev(),
        }
    }

    #[test]
    fn nth() {
        test_iter! {@nth
            [1, 2, 3, 4, 5, 6],
            ds => ds.windows(3).unwrap(),
            s => s.windows(3),
        }

        test_iter! {@nth
            [1, 2, 3, 4, 5],
            ds => ds.windows(3).unwrap(),
            s => s.windows(3),
        }
    }

    #[test]
    fn nth_back() {
        test_iter! {@nth
            [1, 2, 3, 4, 5, 6],
            ds => ds.windows(3).unwrap().rev(),
            s => s.windows(3).rev(),
        }

        test_iter! {@nth
            [1, 2, 3, 4, 5],
            ds => ds.windows(3).unwrap().rev(),
            s => s.windows(3).rev(),
        }
    }
}
