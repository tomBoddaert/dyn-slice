use core::{
    iter::FusedIterator,
    mem::transmute,
    ptr::{metadata, DynMetadata, Pointee},
};

use crate::DynSlice;

/// Dyn slice iterator
pub struct Iter<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>> {
    pub(crate) slice: DynSlice<'a, Dyn>,
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> Clone for Iter<'a, Dyn> {
    fn clone(&self) -> Self {
        Self { slice: self.slice }
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> Iterator for Iter<'a, Dyn> {
    type Item = &'a Dyn;

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            // SAFETY:
            // As the slice is not empty, it must have a first element and a valid vtable pointer, which
            // can be transmuted to `DynMetadata<Dyn>`.
            // The data is guaranteed to live for at least 'a, and not have a mutable reference to it
            // in that time, so the lifetime can be extended.
            let element: &'a Dyn = unsafe { transmute(self.slice.first_unchecked()) };

            // SAFETY:
            // As the slice is not empty, incrementing the pointer by one unit of the underlying type will
            // yield either a valid pointer of the next element, or will yield a pointer one byte after the
            // last element, which is valid as per [`core::ptr::const_ptr::add`]'s safety section.
            self.slice.data = unsafe { self.slice.data.byte_add(metadata(element).size_of()) };
            self.slice.len -= 1;

            Some(element)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.slice.len();
        (remaining, Some(remaining))
    }

    #[inline]
    fn count(self) -> usize {
        self.slice.len()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.slice.len() {
            self.slice.len = 0;
            return None;
        }

        // SAFETY:
        // The above conditional guarantees that the slice is not empty and therefore has a valid vtable
        // pointer, which can be transmuted to a `DynMetadata<Dyn>`.
        let metadata: DynMetadata<Dyn> = unsafe { transmute(self.slice.vtable_ptr) };

        // SAFETY:
        // As `n < slice.len()`, adding `n` units of the underlying type to the pointer will yield a valid
        // pointer in the slice.
        self.slice.data = unsafe { self.slice.data.byte_add(metadata.size_of() * n) };
        self.slice.len -= n;

        self.next()
    }

    fn last(self) -> Option<Self::Item> {
        // SAFETY:
        // The data is guaranteed to live for at least 'a, and not have a mutable reference to it
        // in that time, so the lifetime can be extended.
        unsafe { transmute(self.slice.last()) }
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> DoubleEndedIterator
    for Iter<'a, Dyn>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let element: &'a Dyn =
                // SAFETY:
                // As the slice is not empty, it must have a last element (at `slice.len() - 1`) and a valid
                // vtable pointer, which can be transmuted to `DynMetadata<Dyn>`.
                // The data is guaranteed to live for at least 'a, and not have a mutable reference to it
                // in that time, so the lifetime can be extended.
                unsafe { transmute(self.slice.get_unchecked(self.slice.len - 1)) };

            self.slice.len -= 1;

            Some(element)
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.slice.len() {
            self.slice.len = 0;
            return None;
        }

        self.slice.len -= n;

        self.next_back()
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> ExactSizeIterator
    for Iter<'a, Dyn>
{
    #[inline]
    fn len(&self) -> usize {
        self.slice.len()
    }
}

impl<'a, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + 'a> FusedIterator for Iter<'a, Dyn> {}

#[cfg(test)]
mod test {
    use crate::standard::partial_eq;

    #[test]
    fn test_next() {
        let array = [2, 3, 5, 7, 11];
        let slice = partial_eq::new::<u8, _>(&array);

        let mut iter = slice.iter();
        for &expected in &array {
            let actual = iter.next().expect("expected an element");
            assert!(actual == &expected, "expected {expected}");
        }
    }

    #[test]
    fn test_size_hint() {
        let array = [2, 3, 5, 7, 11];
        let slice = partial_eq::new::<u8, _>(&array);

        let mut iter = slice.iter();
        for expected in (1..=array.len()).rev() {
            let (lower, Some(upper)) = iter.size_hint() else {
                panic!("expected an upper bound");
            };

            assert_eq!(lower, upper, "expected lower and upper bounds to be equal");
            assert_eq!(
                lower, expected,
                "expected size hint to be {expected}, got {lower}"
            );

            let _ = iter.next().expect("expected an element");
        }

        let (lower, Some(upper)) = iter.size_hint() else {
            panic!("expected an upper bound");
        };

        assert_eq!(lower, upper, "expected lower and upper bounds to be equal");
        assert_eq!(lower, 0, "expected size hint to be 0, got {lower}");
    }

    #[test]
    fn test_count() {
        let array = [2, 3, 5, 7, 11];
        let slice = partial_eq::new::<u8, _>(&array);

        let mut iter = slice.iter();
        for expected in (1..=array.len()).rev() {
            let actual = iter.clone().count();
            assert_eq!(
                actual, expected,
                "expected count to be {expected}, got {actual}"
            );

            let _ = iter.next().expect("expected an element");
        }

        let actual = iter.count();
        assert_eq!(actual, 0, "expected count to be 0, got {actual}");
    }

    #[test]
    fn test_nth() {
        let array = [2, 3, 5, 7, 11];
        let slice = partial_eq::new::<u8, _>(&array);

        let mut iter = slice.iter();

        #[allow(clippy::iter_nth_zero)]
        let actual = iter.nth(0).expect("expected an element");
        assert!(actual == &2, "expected 2");

        assert!(
            iter.nth(1).expect("expected an element") == &5,
            "expected 5"
        );
        assert_eq!(iter.size_hint().0, 2, "expected 2 elements left");

        assert!(iter.nth(2).is_none(), "expected none");
        assert_eq!(iter.size_hint().0, 0, "expected 0 elements left");
    }

    #[test]
    fn test_last() {
        let array = [2, 3, 5, 7, 11];
        let slice = partial_eq::new::<u8, _>(&array);

        assert!(
            slice.iter().last().expect("expected an element") == &11,
            "expected 11"
        );
    }

    #[test]
    fn test_next_back() {
        let array = [2, 3, 5, 7, 11];
        let slice = partial_eq::new::<u8, _>(&array);

        let mut iter = slice.iter();
        for &expected in array.iter().rev() {
            let actual = iter.next_back().expect("expected an element");
            assert!(actual == &expected, "expected {expected}");
        }
    }

    #[test]
    fn test_nth_back() {
        let array = [2, 3, 5, 7, 11];
        let slice = partial_eq::new::<u8, _>(&array);

        let mut iter = slice.iter();

        #[allow(clippy::iter_nth_zero)]
        let actual = iter.nth_back(0).expect("expected an element");
        assert!(actual == &11, "expected 11");

        assert!(
            iter.nth_back(1).expect("expected an element") == &5,
            "expected 5"
        );
        assert_eq!(iter.size_hint().0, 2, "expected 2 elements left");

        assert!(iter.nth_back(2).is_none(), "expected none");
        assert_eq!(iter.size_hint().0, 0, "expected 0 elements left");
    }

    #[test]
    fn test_bidirectional() {
        let array = [2, 3, 5, 7, 11];
        let slice = partial_eq::new::<u8, _>(&array);

        let mut iter = slice.iter();

        assert!(
            iter.next().expect("expected an element") == &2,
            "expected 2"
        );
        assert_eq!(iter.size_hint().0, 4, "expected 4 elements left");

        assert!(
            iter.next_back().expect("expected an element") == &11,
            "expected 11"
        );
        assert_eq!(iter.size_hint().0, 3, "expected 3 elements left");

        assert!(
            iter.nth(1).expect("expected an element") == &5,
            "expected 5"
        );
        assert_eq!(iter.size_hint().0, 1, "expected 1 element left");

        assert!(
            iter.nth_back(0).expect("expected an element") == &7,
            "expected 7"
        );
        assert_eq!(iter.size_hint().0, 0, "expected 0 elements left");
    }
}
