use core::{
    any::Any,
    borrow::{Borrow, BorrowMut},
    cmp::{PartialEq, PartialOrd},
    fmt::{
        self, Binary, Debug, Display, LowerExp, LowerHex, Octal, Pointer, UpperExp, UpperHex, Write,
    },
    ops::{
        AddAssign, BitAndAssign, BitOrAssign, BitXorAssign, DivAssign, MulAssign, RemAssign,
        ShlAssign, ShrAssign, SubAssign,
    },
    ptr::{DynMetadata, Pointee},
};

use crate::DynSliceMut;

use super::{declare_new_fn, DynSlice};

declare_new_fn!(Any, pub any);
macro_rules! impl_any_methods {
    ( $( $t:ty ),* ) => {
        $(
            impl<'a> DynSlice<'a, $t> {
                /// Returns `true` if the underlying slice is of type `T`.
                #[must_use]
                pub fn is<T: 'static>(&self) -> bool {
                    self.get(0).map_or(true, <$t>::is::<T>)
                }

                /// Returns the underlying slice as `&[T]`, or `None` if the underlying slice is not of type `T`.
                #[must_use]
                pub fn downcast<T: 'static>(&self) -> Option<&[T]> {
                    unsafe { self.is::<T>().then(|| self.downcast_unchecked()) }
                }
            }
            impl<'a> DynSliceMut<'a, $t> {
                /// Returns the underlying slice as `&mut [T]`, or `None` if the underlying slice is not of type `T`.
                #[must_use]
                pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut [T]> {
                    unsafe { self.is::<T>().then(|| self.downcast_unchecked_mut()) }
                }
            }
        )*
    };
}
impl_any_methods!(dyn Any, dyn Any + Send, dyn Any + Sync + Send);

declare_new_fn!(<T>, AsRef:<T>, pub as_ref);
declare_new_fn!(<T>, AsMut:<T>, pub as_mut);

declare_new_fn!(<Borrowed>, Borrow:<Borrowed>, pub borrow);
declare_new_fn!(<Borrowed>, BorrowMut:<Borrowed>, pub borrow_mut);

declare_new_fn!(<Rhs>, PartialEq:<Rhs>, pub partial_eq);
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + PartialEq<Rhs> + ?Sized, Rhs> PartialEq<[Rhs]>
    for DynSlice<'a, Dyn>
{
    fn eq(&self, other: &[Rhs]) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + PartialEq<Rhs> + ?Sized, Rhs> PartialEq<[Rhs]>
    for DynSliceMut<'a, Dyn>
{
    #[inline]
    fn eq(&self, other: &[Rhs]) -> bool {
        self.0.eq(other)
    }
}

declare_new_fn!(<Rhs>, PartialOrd:<Rhs>, pub partial_ord);

declare_new_fn!(Binary, pub binary);

declare_new_fn!(Debug, pub debug);
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + Debug + ?Sized> Debug for DynSlice<'a, Dyn> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + Debug + ?Sized> Debug
    for DynSliceMut<'a, Dyn>
{
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <DynSlice<Dyn> as Debug>::fmt(&self.0, f)
    }
}

declare_new_fn!(Display, pub display);
declare_new_fn!(LowerExp, pub lower_exp);
declare_new_fn!(LowerHex, pub lower_hex);
declare_new_fn!(Octal, pub octal);

declare_new_fn!(Pointer, pub pointer);
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + ?Sized> Pointer for DynSlice<'a, Dyn> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <*const () as Pointer>::fmt(&self.data, f)
    }
}

declare_new_fn!(UpperExp, pub upper_exp);
declare_new_fn!(UpperHex, pub upper_hex);
declare_new_fn!(Write, pub write);

declare_new_fn!(<Rhs>, AddAssign:<Rhs>, pub add_assign);
declare_new_fn!(<Rhs>, BitAndAssign:<Rhs>, pub bit_and_assign);
declare_new_fn!(<Rhs>, BitOrAssign:<Rhs>, pub bit_or_assign);
declare_new_fn!(<Rhs>, BitXorAssign:<Rhs>, pub bit_xor_assign);
declare_new_fn!(<Rhs>, DivAssign:<Rhs>, pub div_assign);
declare_new_fn!(<Rhs>, MulAssign:<Rhs>, pub mul_assign);
declare_new_fn!(<Rhs>, RemAssign:<Rhs>, pub rem_assign);
declare_new_fn!(<Rhs>, ShlAssign:<Rhs>, pub shl_assign);
declare_new_fn!(<Rhs>, ShrAssign:<Rhs>, pub shr_assign);
declare_new_fn!(<Rhs>, SubAssign:<Rhs>, pub sub_assign);

/// A reference-to-value conversion.
pub trait To<T> {
    /// Converts this reference into the (usually inferred) input type.
    fn to(&self) -> T;
}

// From implies Into, so Into is used to include both traits
impl<T, F: Into<T> + Copy> To<T> for F {
    #[inline]
    fn to(&self) -> T {
        (*self).into()
    }
}

declare_new_fn!(<T>, To:<T>, pub to);

#[cfg(feature = "alloc")]
mod standard_alloc {
    extern crate alloc;
    use alloc::string::ToString;

    use crate::declare_new_fn;

    declare_new_fn!(
        #[cfg_attr(doc, doc(cfg(feature = "alloc")))]
        #[doc = concat!("(only available with the [`alloc` feature](https://docs.rs/crate/dyn-slice/", env!("CARGO_PKG_VERSION"),"/features))")]
        ToString,
        pub to_string
    );
}
#[cfg(feature = "alloc")]
pub use standard_alloc::*;

#[cfg(feature = "std")]
mod standard_std {
    use std::{
        error::Error,
        io::{Seek, Write},
    };

    use crate::declare_new_fn;

    declare_new_fn!(
        #[cfg_attr(doc, doc(cfg(feature = "std")))]
        #[doc = concat!("(only available with the [`std` feature](https://docs.rs/crate/dyn-slice/", env!("CARGO_PKG_VERSION"),"/features))")]
        Error,
        pub error,
    );
    // impl_debug!(dyn Error);

    declare_new_fn!(
        #[cfg_attr(doc, doc(cfg(feature = "std")))]
        #[doc = concat!("(only available with the [`std` feature](https://docs.rs/crate/dyn-slice/", env!("CARGO_PKG_VERSION"),"/features))")]
        Seek,
        pub seek,
    );
    declare_new_fn!(
        #[cfg_attr(doc, doc(cfg(feature = "std")))]
        #[doc = concat!("(only available with the [`std` feature](https://docs.rs/crate/dyn-slice/", env!("CARGO_PKG_VERSION"),"/features))")]
        Write,
        pub io_write,
    );
}
#[cfg(feature = "std")]
pub use standard_std::*;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_any() {
        #[derive(Debug, PartialEq)]
        struct A;

        let array = [A, A];
        let slice = any::new(&array);

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
        let slice = any::new(&array);

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
        let slice = as_ref::new::<u8, _>(&array);

        for (i, y) in array.iter().enumerate() {
            assert_eq!(slice.get(i).expect("expected an element").as_ref(), &**y);
        }
    }

    #[test]
    fn test_borrow() {
        let a: Box<u8> = Box::new(5);
        let b: Box<u8> = Box::new(7);

        let array = [a, b];
        let slice = borrow::new::<u8, _>(&array);

        for (i, y) in array.iter().enumerate() {
            assert_eq!(slice.get(i).expect("expected an element").borrow(), &**y);
        }
    }

    #[test]
    fn test_partial_eq() {
        let array: [u8; 2] = [5, 7];
        let slice = partial_eq::new::<u8, _>(&array);

        for (i, y) in array.iter().enumerate() {
            let element = slice.get(i).expect("expected an element");
            assert!(element == y);
            assert!(element != &200);
        }
    }

    #[test]
    fn test_partial_ord() {
        let array: [u8; 2] = [5, 7];
        let slice = partial_ord::new::<u8, _>(&array);

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
        let slice = debug::new(&array);

        for i in 0..array.len() {
            let element = slice.get(i).expect("expected an element");
            assert_eq!(format!("{element:?}"), debugged);
        }

        assert_eq!(format!("{slice:?}"), format!("{array:?}"));

        let slice = debug::new::<A>(&[]);
        assert_eq!(format!("{slice:?}"), "[]");

        let array = [A];
        let slice = debug::new(&array);
        assert_eq!(format!("{slice:?}"), format!("{array:?}"));
    }

    #[test]
    fn test_display() {
        struct A;
        impl fmt::Display for A {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A displayed")
            }
        }
        let displayed = format!("{A}");

        let array = [A, A];
        let slice = display::new(&array);

        for i in 0..array.len() {
            let element = slice.get(i).expect("expected an element");
            assert_eq!(format!("{element}"), displayed);
        }
    }

    #[test]
    fn test_to() {
        use core::num::NonZeroU8;

        let a: u8 = 5;
        let b: u8 = <u8 as To<u8>>::to(&a);

        assert_eq!(a, b);

        let b: u16 = <u8 as To<u16>>::to(&a);
        let a: u16 = a.into();

        assert_eq!(a, b);

        let array: [NonZeroU8; 2] =
            unsafe { [NonZeroU8::new_unchecked(5), NonZeroU8::new_unchecked(7)] };
        let slice = to::new::<u8, _>(&array);

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
        let slice = to_string::new(&array);

        for i in 0..array.len() {
            let element = slice.get(i).expect("expected an element");
            assert_eq!(element.to_string(), displayed);
        }
    }

    #[test]
    fn test_error() {
        #[derive(Debug)]
        struct A;
        impl fmt::Display for A {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "A displayed")
            }
        }
        impl std::error::Error for A {}
        let displayed = format!("{A}");

        let array = [A, A];
        let slice = error::new(&array);

        for i in 0..array.len() {
            let element = slice.get(i).expect("expected an element");
            assert_eq!(format!("{element}"), displayed);
        }

        assert_eq!(format!("{slice:?}"), format!("{array:?}"));
    }
}
