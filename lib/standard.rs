use core::{
    any::Any,
    borrow::{Borrow, BorrowMut},
    cmp::{PartialEq, PartialOrd},
    convert::{AsMut, AsRef},
    fmt::{
        self, Binary, Debug, Display, LowerExp, LowerHex, Octal, Pointer, UpperExp, UpperHex, Write,
    },
    future::Future,
    hash::{self, BuildHasher, Hasher},
    iter::{DoubleEndedIterator, ExactSizeIterator, FusedIterator, Iterator},
    marker::{Send, Sized, Sync},
    ops::{
        AddAssign, BitAndAssign, BitOrAssign, BitXorAssign, Deref, DerefMut, DivAssign, Index,
        IndexMut, MulAssign, RemAssign, ShlAssign, ShrAssign, SubAssign,
    },
    ptr::{DynMetadata, Pointee},
};

use crate::DynSliceMut;

use super::{declare_new_fns, DynSlice};

#[allow(unused)]
macro_rules! feature_availability {
    ( $feature:literal ) => {
        concat!(
            "(only available with the [`",
            $feature,
            "` feature](https://docs.rs/crate/dyn-slice/",
            env!("CARGO_PKG_VERSION"),
            "/features))"
        )
    };
}

declare_new_fns!(
    #[crate = crate]
    ///
    /// `DynSlice(Mut)<dyn Any>`, `DynSlice(Mut)<dyn Any + Send>` and `DynSlice(Mut)<dyn Any + Send + Sync>` have a few extra methods:
    /// - [`DynSlice::is`]
    /// - [`DynSlice::downcast`]
    /// - [`DynSliceMut::downcast_mut`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyn_slice::standard::any;
    /// let array: [u8; 4] = [1, 2, 4, 8];
    /// let slice = any::new(&array);
    ///
    /// // Assert that the dyn-slice is a slice of `u8`s
    /// assert!(slice.is::<u8>());
    /// // Downcast the dyn-slice to a slice of `u8`s
    /// assert_eq!(slice.downcast::<u8>(), Some(array.as_slice()));
    /// ```
    ///
    /// ```
    /// # use dyn_slice::standard::any;
    /// let mut array: [u8; 4] = [1, 2, 4, 8];
    /// let mut slice = any::new_mut(&mut array);
    ///
    /// // Downcast the mutable dyn-slice to a mutable slice of `u8`s
    /// slice.downcast_mut::<u8>().unwrap()[1] = 255;
    /// assert_eq!(array, [1, 255, 4, 8]);
    /// ```
    pub any Any
);
declare_new_fns!(
    #[crate = crate]
    ///
    /// `DynSlice(Mut)<dyn Any>`, `DynSlice(Mut)<dyn Any + Send>` and `DynSlice(Mut)<dyn Any + Send + Sync>` have a few extra methods:
    /// - [`DynSlice::is`]
    /// - [`DynSlice::downcast`]
    /// - [`DynSliceMut::downcast_mut`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyn_slice::standard::any_send;
    /// let array: [u8; 4] = [1, 2, 4, 8];
    /// let slice = any_send::new(&array);
    ///
    /// // Assert that the dyn-slice is a slice of `u8`s
    /// assert!(slice.is::<u8>());
    /// // Downcast the dyn-slice to a slice of `u8`s
    /// assert_eq!(slice.downcast::<u8>(), Some(array.as_slice()));
    /// ```
    ///
    /// ```
    /// # use dyn_slice::standard::any_send;
    /// let mut array: [u8; 4] = [1, 2, 4, 8];
    /// let mut slice = any_send::new_mut(&mut array);
    ///
    /// // Downcast the mutable dyn-slice to a mutable slice of `u8`s
    /// slice.downcast_mut::<u8>().unwrap()[1] = 255;
    /// assert_eq!(array, [1, 255, 4, 8]);
    /// ```
    pub any_send Any + Send
);
declare_new_fns!(
    #[crate = crate]
    ///
    /// `DynSlice(Mut)<dyn Any>`, `DynSlice(Mut)<dyn Any + Send>` and `DynSlice(Mut)<dyn Any + Send + Sync>` have a few extra methods:
    /// - [`DynSlice::is`]
    /// - [`DynSlice::downcast`]
    /// - [`DynSliceMut::downcast_mut`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyn_slice::standard::any_sync_send;
    /// let array: [u8; 4] = [1, 2, 4, 8];
    /// let slice = any_sync_send::new(&array);
    ///
    /// // Assert that the dyn-slice is a slice of `u8`s
    /// assert!(slice.is::<u8>());
    /// // Downcast the dyn-slice to a slice of `u8`s
    /// assert_eq!(slice.downcast::<u8>(), Some(array.as_slice()));
    /// ```
    ///
    /// ```
    /// # use dyn_slice::standard::any_sync_send;
    /// let mut array: [u8; 4] = [1, 2, 4, 8];
    /// let mut slice = any_sync_send::new_mut(&mut array);
    ///
    /// // Downcast the mutable dyn-slice to a mutable slice of `u8`s
    /// slice.downcast_mut::<u8>().unwrap()[1] = 255;
    /// assert_eq!(array, [1, 255, 4, 8]);
    /// ```
    pub any_sync_send Any + Sync + Send
);
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
                    self.is::<T>().then(|| {
                        // SAFETY:
                        // The above line guarantees that the underlying slice is of type `T`,
                        // so the downcast is valid.
                        unsafe { self.downcast_unchecked() }
                    })
                }
            }

            impl<'a> DynSliceMut<'a, $t> {
                /// Returns the underlying slice as `&mut [T]`, or `None` if the underlying slice is not of type `T`.
                #[must_use]
                pub fn downcast_mut<T: 'static>(&mut self) -> Option<&mut [T]> {
                    self.0.is::<T>().then(|| {
                        // SAFETY:
                        // The above line guarantees that the underlying slice is of type `T`,
                        // so the downcast is valid.
                        unsafe { self.downcast_unchecked_mut() }
                    })
                }
            }
        )*
    };
}
impl_any_methods!(dyn Any, dyn Any + Send, dyn Any + Sync + Send);

declare_new_fns!(
    #[crate = crate]
    pub borrow<Borrowed> Borrow<Borrowed>
);
declare_new_fns!(
    #[crate = crate]
    pub borrow_mut<Borrowed> BorrowMut<Borrowed>
);

declare_new_fns!(
    #[crate = crate]
    ///
    /// `DynSlice(Mut)<dyn PartialEq<Rhs>>` implements `PartialEq<[Rhs]>`
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyn_slice::standard::partial_eq;
    /// let array: [u8; 4] = [1, 2, 4, 8];
    /// let slice = partial_eq::new(&array);
    ///
    /// assert!(slice == array.as_slice());
    /// ```
    pub partial_eq<Rhs> PartialEq<Rhs>
);
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
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + PartialEq<Rhs> + ?Sized, Rhs> PartialEq<&[Rhs]>
    for DynSlice<'a, Dyn>
{
    #[inline]
    fn eq(&self, other: &&[Rhs]) -> bool {
        self.eq(*other)
    }
}
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + PartialEq<Rhs> + ?Sized, Rhs> PartialEq<&[Rhs]>
    for DynSliceMut<'a, Dyn>
{
    #[inline]
    fn eq(&self, other: &&[Rhs]) -> bool {
        self.0.eq(*other)
    }
}
declare_new_fns!(
    #[crate = crate]
    pub partial_ord<Rhs> PartialOrd<Rhs>
);

declare_new_fns!(
    #[crate = crate]
    pub as_ref<T> AsRef<T>
);
declare_new_fns!(
    #[crate = crate]
    pub as_mut<T> AsMut<T>
);

declare_new_fns!(
    #[crate = crate]
    pub binary Binary
);
declare_new_fns!(
    #[crate = crate]
    ///
    /// # Examples
    ///
    /// ```
    /// # use dyn_slice::standard::debug;
    /// let array: [u8; 4] = [1, 2, 4, 8];
    /// let slice = debug::new(&array);
    ///
    /// assert_eq!(
    ///     format!("{slice:?}"),
    ///     "[1, 2, 4, 8]",
    /// );
    /// ```
    pub debug Debug
);
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
declare_new_fns!(
    #[crate = crate]
    pub display Display
);
declare_new_fns!(
    #[crate = crate]
    pub lower_exp LowerExp
);
declare_new_fns!(
    #[crate = crate]
    pub lower_hex LowerHex
);
declare_new_fns!(
    #[crate = crate]
    pub octal Octal
);
declare_new_fns!(
    #[crate = crate]
    pub pointer Pointer
);
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + ?Sized> Pointer for DynSlice<'a, Dyn> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <*const () as Pointer>::fmt(&self.data, f)
    }
}
impl<'a, Dyn: Pointee<Metadata = DynMetadata<Dyn>> + ?Sized> Pointer for DynSliceMut<'a, Dyn> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <*const () as Pointer>::fmt(&self.data, f)
    }
}
declare_new_fns!(
    #[crate = crate]
    pub upper_exp UpperExp
);
declare_new_fns!(
    #[crate = crate]
    pub upper_hex UpperHex
);
declare_new_fns!(
    #[crate = crate]
    pub write Write
);

declare_new_fns!(
    #[crate = crate]
    pub future<Output> Future<Output = Output>
);

declare_new_fns!(
    #[crate = crate]
    pub build_hasher<Hasher: hash::Hasher> BuildHasher<Hasher = Hasher>
);
declare_new_fns!(
    #[crate = crate]
    pub hasher Hasher
);

declare_new_fns!(
    #[crate = crate]
    pub double_ended_iterator<Item> DoubleEndedIterator<Item = Item>
);
declare_new_fns!(
    #[crate = crate]
    pub exact_size_iterator<Item> ExactSizeIterator<Item = Item>
);
declare_new_fns!(
    #[crate = crate]
    pub fused_iterator<Item> FusedIterator<Item = Item>
);
declare_new_fns!(
    ///
    /// ```rust
    /// # use dyn_slice::standard::iterator;
    /// let mut array = [(1..5), (2..9), (4..6)];
    /// let mut slice = iterator::new_mut(&mut array);
    ///
    /// assert_eq!(slice[0].next(), Some(1));
    /// assert_eq!(slice[1].next(), Some(2));
    /// assert_eq!(slice[2].next(), Some(4));
    ///
    /// assert_eq!(slice[0].next(), Some(2));
    /// ```
    #[crate = crate]
    pub iterator<Item> Iterator<Item = Item>
);

declare_new_fns!(
    #[crate = crate]
    pub add_assign<Rhs> AddAssign<Rhs>
);
declare_new_fns!(
    #[crate = crate]
    pub bit_and_assign<Rhs> BitAndAssign<Rhs>
);
declare_new_fns!(
    #[crate = crate]
    pub bit_or_assign<Rhs> BitOrAssign<Rhs>
);
declare_new_fns!(
    #[crate = crate]
    pub bit_xor_assign<Rhs> BitXorAssign<Rhs>
);
declare_new_fns!(
    #[crate = crate]
    pub deref<Target> Deref<Target = Target>
);
declare_new_fns!(
    #[crate = crate]
    pub deref_mut<Target> DerefMut<Target = Target>
);
declare_new_fns!(
    #[crate = crate]
    pub div_assign<Rhs> DivAssign<Rhs>
);
declare_new_fns!(
    #[crate = crate]
    pub index<Idx: ?Sized, Output: ?Sized> Index<Idx, Output = Output>
);
declare_new_fns!(
    #[crate = crate]
    pub index_mut<Idx: ?Sized, Output: ?Sized> IndexMut<Idx, Output = Output>
);
declare_new_fns!(
    #[crate = crate]
    pub mul_assign<Rhs> MulAssign<Rhs>
);
declare_new_fns!(
    #[crate = crate]
    pub rem_assign<Rhs> RemAssign<Rhs>
);
declare_new_fns!(
    #[crate = crate]
    pub shl_assign<Rhs> ShlAssign<Rhs>
);
declare_new_fns!(
    #[crate = crate]
    pub shr_assign<Rhs> ShrAssign<Rhs>
);
declare_new_fns!(
    #[crate = crate]
    pub sub_assign<Rhs> SubAssign<Rhs>
);

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

declare_new_fns!(
    #[crate = crate]
    pub to<T> To<T>
);

#[cfg(feature = "alloc")]
mod standard_alloc {
    extern crate alloc;
    use alloc::string::ToString;

    use crate::declare_new_fns;

    declare_new_fns!(
        #[crate = crate]
        #[cfg_attr(doc, doc(cfg(feature = "alloc")))]
        #[doc = feature_availability!("alloc")]
        pub to_string ToString
    );
}
#[cfg(feature = "alloc")]
pub use standard_alloc::*;

#[cfg(feature = "std")]
mod standard_std {
    use std::{
        error::Error,
        io::{BufRead, IsTerminal, Read, Seek, Write},
        net::ToSocketAddrs,
    };

    use crate::declare_new_fns;

    declare_new_fns!(
        #[crate = crate]
        #[cfg_attr(doc, doc(cfg(feature = "std")))]
        #[doc = feature_availability!("std")]
        pub error Error
    );

    declare_new_fns!(
        #[crate = crate]
        #[cfg_attr(doc, doc(cfg(feature = "std")))]
        #[doc = feature_availability!("std")]
        pub buf_read BufRead
    );
    declare_new_fns!(
        #[crate = crate]
        #[cfg_attr(doc, doc(cfg(feature = "std")))]
        #[doc = feature_availability!("std")]
        pub is_terminal IsTerminal
    );
    declare_new_fns!(
        #[crate = crate]
        #[cfg_attr(doc, doc(cfg(feature = "std")))]
        #[doc = feature_availability!("std")]
        pub io_read Read
    );
    declare_new_fns!(
        #[crate = crate]
        #[cfg_attr(doc, doc(cfg(feature = "std")))]
        #[doc = feature_availability!("std")]
        pub seek Seek
    );
    declare_new_fns!(
        #[crate = crate]
        #[cfg_attr(doc, doc(cfg(feature = "std")))]
        #[doc = feature_availability!("std")]
        pub io_write Write
    );

    declare_new_fns!(
        #[crate = crate]
        #[cfg_attr(doc, doc(cfg(feature = "std")))]
        #[doc = feature_availability!("std")]
        pub to_socket_addrs<Iter: core::iter::Iterator<Item = std::net::SocketAddr>>
            ToSocketAddrs<Iter = Iter>
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
    fn test_hasher() {
        use std::collections::hash_map::DefaultHasher;

        const TEST_DATA: &[u8] = b"test hash data";

        let mut reference = DefaultHasher::new();
        reference.write(TEST_DATA);
        let reference = reference.finish();

        let mut array = [DefaultHasher::new(), DefaultHasher::new()];
        let mut slice = hasher::new_mut(&mut array);

        for hasher in &mut slice {
            hasher.write(TEST_DATA);

            assert_eq!(hasher.finish(), reference);
        }
    }

    #[test]
    fn test_iterator() {
        let mut array = [(0..5), (10..15), (-30..-25)];
        let mut slice = iterator::new_mut(&mut array);

        for (range, expected) in slice.iter_mut().zip([0, 10, -30]) {
            assert_eq!(range.next(), Some(expected));
        }

        for (range, expected) in slice.iter_mut().zip([1, 11, -29]) {
            assert_eq!(range.next(), Some(expected));
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

        let array: [NonZeroU8; 2] = {
            // SAFETY:
            // NonZeroU8 has the same layout as u8, and can therefore be transmuted.
            unsafe { [NonZeroU8::new_unchecked(5), NonZeroU8::new_unchecked(7)] }
        };
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
