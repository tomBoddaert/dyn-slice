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
//! There are some pre-made new functions for common traits in [`standard`]

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
mod iter;
mod iter_mut;
/// Dyn slice `new` and `new_mut` definitions for some common traits
///
/// If you want a dyn slice for a trait that is not here, use the [`declare_new_fns`] macro.
pub mod standard;

pub use dyn_slice::*;
pub use dyn_slice_mut::*;
pub use iter::*;
pub use iter_mut::*;

/// Declare `new` and `new_mut` functions for dyn slices of a trait
///
/// # Syntax
/// ```text
/// declare_new_fns!(
///     #[attributes]
///     pub name<generics> Trait<parameters>
///     where
///         generic: bounds,
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
///     display_slice ::std::fmt::Display
/// );
/// ```
///
/// ## Other examples
#[doc = concat!("There are more examples of how to use [`declare_new_fns`] in the [examples directory](https://docs.rs/crate/dyn-slice/", env!("CARGO_PKG_VERSION"), "/source/examples/).")]
///
/// # Use from other crates
/// When using `dyn_slice` from crates that re-export it, you may need to add a crate attribute, for example:
/// ```text
/// declare_new_fns!(
///     #[crate = other_crate::dyn_slice]
///     name Trait
/// );
/// ```
pub use dyn_slice_macros::declare_new_fns;

#[doc(hidden)]
#[deprecated(
    since = "3.2.0",
    note = "this has been replaced with `declare_new_fns`. Convert to the new macro or expand this one"
)]
#[allow(clippy::crate_in_macro_def)]
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
