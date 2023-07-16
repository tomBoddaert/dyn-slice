use core::{
    any::Any,
    borrow::Borrow,
    cmp::{PartialEq, PartialOrd},
    fmt::{Debug, Display},
};

use super::{declare_dyn_slice, DynSliceMethods};

declare_dyn_slice!(Any, any_dyn_slice);
pub use any_dyn_slice::DynSlice as AnyDynSlice;
impl<'a> AnyDynSlice<'a> {
    /// Returns `true` if the underlying slice is of type `T`.
    #[must_use]
    pub fn is<T: 'static>(&self) -> bool {
        self.get(0).map_or(true, <dyn Any>::is::<T>)
    }

    /// Returns the underlying slice as `&[T]`, or `None` if the underlying slice is not of type `T`.
    #[must_use]
    pub fn downcast<T: 'static>(&self) -> Option<&[T]> {
        unsafe { self.is::<T>().then(|| self.downcast_unchecked()) }
    }
}

declare_dyn_slice!(<T>, AsRef:<T>, as_ref_dyn_slice);
pub use as_ref_dyn_slice::DynSlice as AsRefDynSlice;

declare_dyn_slice!(<T>, Borrow:<T>, borrow_dyn_slice);
pub use borrow_dyn_slice::DynSlice as BorrowDynSlice;

declare_dyn_slice!(<Rhs>, PartialEq:<Rhs>, partial_eq_dyn_slice);
pub use partial_eq_dyn_slice::DynSlice as PartialEqDynSlice;
impl<'a, Rhs> PartialEq<[Rhs]> for PartialEqDynSlice<'a, Rhs> {
    fn eq(&self, other: &[Rhs]) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

declare_dyn_slice!(<Rhs>, PartialOrd:<Rhs>, partial_ord_dyn_slice);
pub use partial_ord_dyn_slice::DynSlice as PartialOrdDynSlice;
impl<'a, Rhs> PartialEq<[Rhs]> for PartialOrdDynSlice<'a, Rhs> {
    fn eq(&self, other: &[Rhs]) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

declare_dyn_slice!(Debug, debug_dyn_slice);
pub use debug_dyn_slice::DynSlice as DebugDynSlice;
impl<'a> Debug for DebugDynSlice<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[")?;
        let mut iter = self.iter();
        if let Some(element) = iter.next() {
            element.fmt(f)?;
        }
        for element in iter {
            write!(f, ", ")?;
            element.fmt(f)?;
        }
        write!(f, "]")
    }
}

declare_dyn_slice!(Display, display_dyn_slice);
pub use display_dyn_slice::DynSlice as DisplayDynSlice;

/// A reference-to-value conversion.
pub trait To<T> {
    /// Converts this reference into the (usually inferred) input type.
    fn to(&self) -> T;
}

// From implies Into, so Into is used to include both traits
impl<T, F: Clone + Into<T>> To<T> for F {
    fn to(&self) -> T {
        self.clone().into()
    }
}

declare_dyn_slice!(<T>, To:<T>, to_dyn_slice);
pub use to_dyn_slice::DynSlice as ToDynSlice;

#[cfg(feature = "alloc")]
mod alloc_lib {
    extern crate alloc;
    use alloc::string::ToString;

    use crate::declare_dyn_slice;

    declare_dyn_slice!(
        #[doc = concat!("(only available with the [`alloc` feature](https://docs.rs/crate/dyn-slice/", env!("CARGO_PKG_VERSION"),"/features))")]
        ToString,
        to_string_dyn_slice
    );
    pub use to_string_dyn_slice::DynSlice as ToStringDynSlice;
}
#[cfg(feature = "alloc")]
pub use alloc_lib::*;

#[cfg(feature = "std")]
mod std_lib {
    use std::{error::Error, fmt::Debug};

    use crate::{declare_dyn_slice, DynSliceMethods};

    declare_dyn_slice!(
        #[doc = concat!("(only available with the [`std` feature](https://docs.rs/crate/dyn-slice/", env!("CARGO_PKG_VERSION"),"/features))")]
        Error,
        error_dyn_slice
    );
    pub use error_dyn_slice::DynSlice as ErrorDynSlice;
    impl<'a> Debug for ErrorDynSlice<'a> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "[")?;
            let mut iter = self.iter();
            if let Some(element) = iter.next() {
                Debug::fmt(element, f)?;
            }
            for element in iter {
                write!(f, ", ")?;
                Debug::fmt(element, f)?;
            }
            write!(f, "]")
        }
    }
}
#[cfg(feature = "std")]
pub use std_lib::*;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_any() {
        #[derive(Debug, PartialEq)]
        struct A;

        let array = [A, A];
        let slice = AnyDynSlice::new(&array);

        assert!(slice.is::<A>());
        assert!(!slice.is::<u8>());

        assert_eq!(slice.downcast::<A>(), Some(&array[..]));
        assert_eq!(slice.downcast::<u8>(), None);

        // Using for loop rather than iter to make sure exactly 2 elements are
        // checked, without trusting iter
        for i in 0..array.len() {
            assert!(slice.get(i).expect("expected an element").is::<A>());
        }

        // Make sure the slice can be downcast to anything when empty

        let array: [A; 0] = [];
        let slice = AnyDynSlice::new(&array);

        assert!(slice.is::<A>());
        assert!(slice.is::<u8>());

        assert_eq!(slice.downcast::<A>(), Some(&array[..]));
        assert_eq!(slice.downcast::<u8>(), Some(&[][..]));
    }

    #[test]
    fn test_as_ref() {
        let a: Box<u8> = Box::new(5);
        let b: Box<u8> = Box::new(7);

        let array = [a, b];
        let slice = AsRefDynSlice::<u8>::new(&array);

        for (i, y) in array.iter().enumerate() {
            assert_eq!(slice.get(i).expect("expected an element").as_ref(), &**y);
        }
    }

    #[test]
    fn test_borrow() {
        let a: Box<u8> = Box::new(5);
        let b: Box<u8> = Box::new(7);

        let array = [a, b];
        let slice = BorrowDynSlice::<u8>::new(&array);

        for (i, y) in array.iter().enumerate() {
            assert_eq!(slice.get(i).expect("expected an element").borrow(), &**y);
        }
    }

    #[test]
    fn test_partial_eq() {
        let array: [u8; 2] = [5, 7];
        let slice = PartialEqDynSlice::<u8>::new(&array);

        for (i, y) in array.iter().enumerate() {
            let element = slice.get(i).expect("expected an element");
            assert!(element == y);
            assert!(element != &200);
        }
    }

    #[test]
    fn test_partial_ord() {
        let array: [u8; 2] = [5, 7];
        let slice = PartialOrdDynSlice::<u8>::new(&array);

        for (i, y) in array.iter().enumerate() {
            let element = slice.get(i).expect("expected an element");
            assert!(element > &3);
            assert!(element == y);
            assert!(element < &10);
        }
    }

    #[test]
    fn test_debug() {
        #[derive(Debug)]
        struct A;
        let debugged = format!("{A:?}");

        let array = [A, A];
        let slice = DebugDynSlice::new(&array);

        for i in 0..array.len() {
            let element = slice.get(i).expect("expected an element");
            assert_eq!(format!("{element:?}"), debugged);
        }

        assert_eq!(format!("{slice:?}"), format!("{array:?}"));

        let slice = DebugDynSlice::new::<A>(&[]);
        assert_eq!(format!("{slice:?}"), "[]");

        let array = [A];
        let slice = DebugDynSlice::new(&array);
        assert_eq!(format!("{slice:?}"), format!("{array:?}"));
    }

    #[test]
    fn test_display() {
        struct A;
        impl core::fmt::Display for A {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "A displayed")
            }
        }
        let displayed = format!("{A}");

        let array = [A, A];
        let slice = DisplayDynSlice::new(&array);

        for i in 0..array.len() {
            let element = slice.get(i).expect("expected an element");
            assert_eq!(format!("{element}"), displayed);
        }
    }

    #[test]
    fn test_to() {
        use std::num::NonZeroU8;

        let a: u8 = 5;
        let b: u8 = <u8 as To<u8>>::to(&a);

        assert_eq!(a, b);

        let b: u16 = <u8 as To<u16>>::to(&a);
        let a: u16 = a.into();

        assert_eq!(a, b);

        let array: [NonZeroU8; 2] =
            unsafe { [NonZeroU8::new_unchecked(5), NonZeroU8::new_unchecked(7)] };
        let slice = ToDynSlice::<u8>::new(&array);

        for (i, y) in array.iter().enumerate() {
            let element = slice.get(i).expect("expected an element");
            assert_eq!(element.to(), y.get());
        }
    }

    #[test]
    fn test_to_string() {
        struct A;
        impl core::fmt::Display for A {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "A displayed")
            }
        }
        let displayed = A.to_string();

        let array = [A, A];
        let slice = ToStringDynSlice::new(&array);

        for i in 0..array.len() {
            let element = slice.get(i).expect("expected an element");
            assert_eq!(element.to_string(), displayed);
        }
    }

    #[test]
    fn test_error() {
        #[derive(Debug)]
        struct A;
        impl core::fmt::Display for A {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "A displayed")
            }
        }
        impl std::error::Error for A {}
        let displayed = format!("{A}");

        let array = [A, A];
        let slice = ErrorDynSlice::new(&array);

        for i in 0..array.len() {
            let element = slice.get(i).expect("expected an element");
            assert_eq!(format!("{element}"), displayed);
        }

        assert_eq!(format!("{slice:?}"), format!("{array:?}"));
    }
}
