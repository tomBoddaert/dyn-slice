//! An implementation for a `&dyn [Trait]`-like type, inspired by a [Reddit thread](https://www.reddit.com/r/rust/comments/14i08gz/dyn_slices).
//!
//! Indexing into a dyn slice yields a dyn object.
//!
//! # Examples
//!
//! ```
#![doc = include_str!("../examples/display.rs")]
//! ```
//!
#![doc = concat!("There are more examples in the [examples directory](https://docs.rs/crate/dyn-slice/", env!("CARGO_PKG_VERSION"), "/source/examples/).")]
//!
//! # Standard new dyn slice functions
//!
//! There are some pre-made new functions for common traits in [`standard`].

#![feature(ptr_metadata, pointer_byte_offsets)]
#![cfg_attr(doc, feature(doc_cfg))]
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::perf,
    clippy::cargo,
    clippy::alloc_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    clippy::get_unwrap,
    clippy::panic_in_result_fn,
    clippy::pub_without_shorthand,
    clippy::redundant_type_annotations,
    clippy::todo,
    clippy::undocumented_unsafe_blocks
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod compile_tests;
mod dyn_slice;
mod dyn_slice_mut;
/// Iterator types.
pub mod iter;
/// Dyn slice `new` and `new_mut` definitions for some common traits.
///
/// If you want a dyn slice for a trait that is not here, use the [`declare_new_fns`] macro.
pub mod standard;
mod utils;

pub use dyn_slice::*;
pub use dyn_slice_mut::*;
pub use iter::{Iter, IterMut};

/// Declare `new` and `new_mut` functions for dyn slices of a trait.
///
/// # Syntax
/// ```text
/// declare_new_fns!(
///     #[attributes]
///     pub name<parameters> Trait<arguments>
///     where
///         parameter: bounds,
/// );
/// ```
///
/// The [`ptr_metadata`](https://doc.rust-lang.org/beta/unstable-book/library-features/ptr-metadata.html)
/// feature must be enabled to use this macro!
///
/// ## Example: Display
/// ```
/// #![feature(ptr_metadata)]
/// # use dyn_slice::declare_new_fns;
/// declare_new_fns!(
///     display_slice std::fmt::Display
/// );
/// ```
///
/// ## Other examples
#[doc = concat!("There are more examples of how to use [`declare_new_fns`] in the [examples directory](https://docs.rs/crate/dyn-slice/", env!("CARGO_PKG_VERSION"), "/source/examples/).")]
///
/// # Use from other crates
/// When using `dyn_slice` from crates that re-export it, you may need to add a `crate` attribute, for example:
/// ```text
/// declare_new_fns!(
///     #[crate = other_crate::dyn_slice]
///     name Trait
/// );
/// ```
pub use dyn_slice_macros::declare_new_fns;

#[deprecated(
    since = "3.2.0",
    note = "this has been replaced with `declare_new_fns`. Convert to the new macro or expand this one"
)]
#[macro_export]
/// DEPRECATED, use [`declare_new_fns`] instead!
///
/// Declare `new` and `new_mut` functions for dyn slices of a trait
macro_rules! declare_new_fn {
    (
        $(#[ $meta:meta ])*
        $(< $( $gen:ident ),* >,)?
        $tr:path $( :<$( $trgen:ident ),*> )? $(:+ $atr:path )*,
        $vis:vis $name:ident $(,)?
    ) => {
        $crate::declare_new_fns!(
            $(#[ $meta ])*
            $vis $name $(< $( $gen ),* >)?
                $tr $(< $( $trgen ),* >)?
        );
    };
}

#[cfg(test)]
mod test {
    use core::fmt;

    use dyn_slice_macros::declare_new_fns;

    pub trait Ped<Rhs>: PartialEq<Rhs> + fmt::Debug {}
    impl<T, Rhs> Ped<Rhs> for T where T: PartialEq<Rhs> + fmt::Debug {}

    declare_new_fns! {
        #[crate = crate]
        pub ped<Rhs> Ped<Rhs>
    }

    macro_rules! test_iter {
        (
            $a:expr,
            $ds:ident => $dsiter:expr,
            $s:ident => $siter:expr,
        ) => {
            let a = $a;
            let $ds = ped::new::<u8, u8>(&a);

            let mut iter = $dsiter;
            let mut expected_iter = {
                let $s: &[u8] = &a;
                $siter
            };
            assert_eq!(
                iter.len(),
                expected_iter.len(),
                "initial length was not equal to expected initial length"
            );

            while let Some(expected) = expected_iter.next() {
                let actual = iter.next().expect("expected another item");
                assert_eq!(actual, expected, "item was not equal to expected item");

                assert_eq!(
                    iter.len(),
                    expected_iter.len(),
                    "length was not equal to expected length"
                );
            }

            assert_eq!(iter.len(), 0, "length was not zero");
            assert!(iter.next().is_none(), "expected no more elements");
        };

        (
            mut $a:expr,
            $ds:ident => $dsiter:expr,
            $s:ident => $siter:expr,
        ) => {
            let a = $a;
            let mut a_mut = a;
            let mut $ds = ped::new_mut::<u8, u8>(&mut a_mut);

            let mut iter = $dsiter;
            let mut expected_iter = {
                let $s: &[u8] = &a;
                $siter
            };
            assert_eq!(
                iter.len(),
                expected_iter.len(),
                "initial length was not equal to expected initial length"
            );

            while let Some(expected) = expected_iter.next() {
                let actual = iter.next().expect("expected another item");
                assert_eq!(actual, expected, "item was not equal to expected item");

                assert_eq!(
                    iter.len(),
                    expected_iter.len(),
                    "length was not equal to expected length"
                );
            }

            assert_eq!(iter.len(), 0, "length was not zero");
            assert!(iter.next().is_none(), "expected no more elements");
        };

        (@nth
            $a:expr,
            $ds:ident => $dsiter:expr,
            $s:ident => $siter:expr,
        ) => {
            let a = $a;
            let $ds = ped::new::<u8, u8>(&a);

            let len = {
                let $s: &[u8] = &a;
                $siter.len()
            };

            for n in 0..len {
                let mut iter = $dsiter;
                let mut expected_iter = {
                    let $s: &[u8] = &a;
                    $siter
                };

                let expected = expected_iter
                    .nth(n)
                    .expect("This is a bug in the test: expected an item from the oracle iterator");
                let actual = iter.nth(n).expect("expected an item");

                assert_eq!(actual, expected, "item was not equal to expected item");

                assert_eq!(
                    iter.len(),
                    expected_iter.len(),
                    "length was not equal to expected length"
                );
            }

            let mut iter = $dsiter;
            assert!(iter.nth(len).is_none(), "expected no more elements");
            assert_eq!(iter.len(), 0, "length was not zero");
        };

        (@nth
            mut $a:expr,
            $ds:ident => $dsiter:expr,
            $s:ident => $siter:expr,
        ) => {
            let a = $a;
            let mut a_mut = a;
            let mut $ds = ped::new_mut::<u8, u8>(&mut a_mut);

            let len = {
                let $s: &[u8] = &a;
                $siter.len()
            };

            for n in 0..len {
                let mut iter = $dsiter;
                let mut expected_iter = {
                    let $s: &[u8] = &a;
                    $siter
                };

                let expected = expected_iter
                    .nth(n)
                    .expect("This is a bug in the test: expected an item from the oracle iterator");
                let actual = iter.nth(n).expect("expected an item");

                assert_eq!(actual, expected, "item was not equal to expected item");

                assert_eq!(
                    iter.len(),
                    expected_iter.len(),
                    "length was not equal to expected length"
                );
            }

            let mut iter = $dsiter;
            assert!(iter.nth(len).is_none(), "expected no more elements");
            assert_eq!(iter.len(), 0, "length was not zero");
        };
    }
    pub(crate) use test_iter;
}
