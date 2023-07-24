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
//! # Standard dyn slice types
//!
//! There are some dyn slice declarations for common traits in [`standard`]

#![feature(ptr_metadata, pointer_byte_offsets)]
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
    clippy::todo
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod compile_tests;
mod dyn_slice;
mod dyn_slice_mut;
mod iter;
mod iter_mut;
/// Dyn slice `new` and `new_mut` definitions for some common traits
pub mod standard;

pub use dyn_slice::*;
pub use dyn_slice_mut::*;
pub use iter::*;
pub use iter_mut::*;

#[macro_export]
/// Declares `new` and `new_mut` functions for a dyn slice
///
/// # Syntax
///
/// Simple trait:  
/// `declare_dyn_slice!(Trait, new_module_name);`
///
/// Trait with a given generic:  
/// `declare_dyn_slice!(Trait:<Type>, new_module_name);`
///
/// Trait with a generic:
/// `declare_dyn_slice!(<T>, Trait:<T>, new_module_name);`
///
/// # Examples
///
/// Display:
/// ```
/// # #![feature(ptr_metadata)]
/// # use dyn_slice::declare_new_fn;
/// declare_new_fn!(std::fmt::Display, display_dyn_slice);
/// use display_dyn_slice::{new, new_mut};
/// ```
///
/// Generics:
/// ```
/// # #![feature(ptr_metadata)]
/// # mod test {
/// # use dyn_slice::declare_new_fn;
///
/// pub trait A<B, C> {
///     fn b(&self) -> B;
///     fn c(&self) -> C;
/// }
///
/// declare_new_fn!(A:<u8, f32>, given_generic_dyn_slice);
///
/// declare_new_fn!(<C>, A:<u8, C>, half_given_generic_dyn_slice);
///
/// declare_new_fn!(<B, C>, A:<B, C>, generic_dyn_slice);
/// # }
/// ```
macro_rules! declare_new_fn {
    (
        $(#[ $meta:meta ])*
        $(< $( $gen:ident ),* >,)?
        $tr:path $( :<$( $trgen:ident ),*> )?,
        $vis:vis $name:ident $(,)?
    ) => {
        #[doc = concat!("new functions for `&dyn [", stringify!($tr), "]`")]
        $vis mod $name {
            #[allow(unused_imports)]
            use super::*;
            use $tr as Trait;

            #[allow(unused)]
            #[must_use]
            /// Create a dyn slice from a slice
            pub fn new<'a, $($( $gen ,)*)? DynSliceFromType: Trait $(< $($trgen),* >)?>(value: &'a [DynSliceFromType]) -> $crate::DynSlice<dyn Trait $(< $($trgen),* >)?>
            where
                dyn Trait $(< $($trgen),* >)?: core::ptr::Pointee<Metadata = core::ptr::DynMetadata<dyn Trait $(< $($trgen),* >)?>>,
                $($(
                    $gen: 'static
                ),*)?
            {
                use core::{
                    mem::transmute,
                    ptr::{metadata, null}
                };

                unsafe {
                    // Get the dyn metadata from the first element of value
                    // If value is empty, the metadata should never be accessed, so set it to a null pointer
                    let vtable_ptr = value.get(0).map_or(
                        null::<()>(),
                        |example| {
                            transmute(metadata(transmute::<
                                _,
                                &'static dyn Trait $(< $($trgen),* >)?
                            >(example as &dyn Trait $(< $($trgen),* >)?)))
                        }
                    );

                    $crate::DynSlice::with_vtable_ptr(value, vtable_ptr)
                }
            }

            #[allow(unused)]
            #[must_use]
            /// Create a mutable dyn slice from a mutable slice
            pub fn new_mut<'a, $($( $gen ,)*)? DynSliceFromType: Trait $(< $($trgen),* >)?>(value: &'a mut [DynSliceFromType]) -> $crate::DynSliceMut<dyn Trait $(< $($trgen),* >)?>
            where
                dyn Trait $(< $($trgen),* >)?: core::ptr::Pointee<Metadata = core::ptr::DynMetadata<dyn Trait $(< $($trgen),* >)?>>,
                $($(
                    $gen: 'static
                ),*)?
            {
                use core::{
                    mem::transmute,
                    ptr::{metadata, null}
                };

                unsafe {
                    // Get the dyn metadata from the first element of value
                    // If value is empty, the metadata should never be accessed, so set it to a null pointer
                    let vtable_ptr = value.get(0).map_or(
                        null::<()>(),
                        |example| {
                            transmute(metadata(transmute::<
                                _,
                                &'static dyn Trait $(< $($trgen),* >)?
                            >(example as &dyn Trait $(< $($trgen),* >)?)))
                        }
                    );

                    $crate::DynSliceMut::with_vtable_ptr(value, vtable_ptr)
                }
            }
        }
    };
}
