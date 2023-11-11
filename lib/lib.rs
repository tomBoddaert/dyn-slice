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
/// `declare_new_fn!(Trait, new_module_name);`
///
/// Trait with a given generic:  
/// `declare_new_fn!(Trait:<Type>, new_module_name);`
///
/// Trait with a generic:  
/// `declare_new_fn!(<T>, Trait:<T>, new_module_name);`
///
/// Trait with [auto traits](https://doc.rust-lang.org/beta/reference/special-types-and-traits.html#auto-traits):  
/// `declare_new_fn!(Trait :+ AutoTrait, new_module_name)`
///
/// A visibility qualifier can be added before the module name.
/// Attributes can also be added before the other arguments.
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
        $tr:path $( :<$( $trgen:ident ),*> )? $(:+ $atr:path )*,
        $vis:vis $name:ident $(,)?
    ) => {
        #[doc = concat!("new functions for `&dyn [`[`", stringify!($tr), "`]", $( "` + `[`", stringify!($atr), "`]" ,)* "`]`")]
        $(#[ $meta ])*
        $vis mod $name {
            #[allow(unused_imports)]
            use super::*;
            use $tr as __Trait;

            #[doc = concat!("An alias for `dyn `[`", stringify!($tr), "`]", $( "` + `[`", stringify!($atr), "`]" ,)*)]
            pub type Dyn $(< $( $gen ),* >)? = dyn __Trait $(< $( $trgen ),* >)? $(+ $atr )* + 'static;

            #[doc = concat!("An alias for `&dyn [`[`", stringify!($tr), "`]", $( "` + `[`", stringify!($atr), "`]" ,)* "`]`")]
            pub type Slice<'a, $( $( $gen ),* )?> = $crate::DynSlice<'a, Dyn $(< $( $gen ),* >)?>;

            #[doc = concat!("An alias for `&mut dyn [`[`", stringify!($tr), "`]", $( "` + `[`", stringify!($atr), "`]" ,)* "`]`")]
            pub type SliceMut<'a, $( $( $gen ),* )?> = $crate::DynSliceMut<'a, Dyn $(< $( $gen ),* >)?>;

            $crate::__new_fn!(@new
                | Dyn $(< $( $gen ),* >)?
                | Slice $(< $( $gen ),* >)?
                | new
                | $($( $gen ),*)?
                | $tr
                | $(<$( $trgen ),*>)?
                | $( $atr ),*
            );
            $crate::__new_fn!(@new_mut
                | Dyn $(< $( $gen ),* >)?
                | SliceMut $(< $( $gen ),* >)?
                | new_mut
                | $($( $gen ),*)?
                | $tr
                | $(<$( $trgen ),*>)?
                | $( $atr ),*
            );
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __new_fn {
    (@new
        | $dyn_type:ty
        | $output:ty
        | $name:ident
        | $( $gen:ident ),*
        | $tr:path
        | $(< $( $trgen:ident ),* >)?
        | $( $atr:path ),*
    ) => {
        #[allow(unused)]
        #[must_use]
        #[doc = concat!("Create a dyn slice from a slice of a type that implements [`", stringify!($tr), "`]" $(, "` + `[`", stringify!($atr), "`]" )*)]
        pub fn $name<'a, $( $gen ,)* DynSliceFromType: __Trait $(< $( $trgen ),* >)? $(+ $atr )* + 'static>(value: &'a [DynSliceFromType]) -> $output
        where
            $dyn_type: core::ptr::Pointee<Metadata = core::ptr::DynMetadata<$dyn_type>>,
            $( $gen: 'static ),*
        {
            use core::{
                mem::transmute,
                ptr::{metadata, null}
            };

            // SAFETY:
            // DynMetadata contains a single pointer to the vtable, and the layout is the same as *const (),
            // so it can be transmuted.
            unsafe {
                // Get the dyn metadata from the first element of value
                // If value is empty, the metadata should never be accessed, so set it to a null pointer
                let vtable_ptr = value.get(0).map_or(
                    null::<()>(),
                    |example| {
                        transmute(metadata(example as &$dyn_type))
                    }
                );

                $crate::DynSlice::with_vtable_ptr(value, vtable_ptr)
            }
        }
    };

    (@new_mut
        | $dyn_type:ty
        | $output:ty
        | $name:ident
        | $( $gen:ident ),*
        | $tr:path
        | $(< $( $trgen:ident ),* >)?
        | $( $atr:path ),*
    ) => {
        #[allow(unused)]
        #[must_use]
        #[doc = concat!("Create a mutable dyn slice from a mutable slice of a type that implements [`", stringify!($tr), "`]" $(, "` + `[`", stringify!($atr), "`]" )*)]
        pub fn $name<'a, $( $gen ,)* DynSliceFromType: __Trait $(< $( $trgen ),* >)? $(+ $atr )* + 'static>(value: &'a mut [DynSliceFromType]) -> $output
        where
            $dyn_type: core::ptr::Pointee<Metadata = core::ptr::DynMetadata<$dyn_type>>,
            $( $gen: 'static ),*
        {
            use core::{
                mem::transmute,
                ptr::{metadata, null}
            };

            // SAFETY:
            // DynMetadata contains a single pointer to the vtable, and the layout is the same as *const (),
            // so it can be transmuted.
            unsafe {
                // Get the dyn metadata from the first element of value
                // If value is empty, the metadata should never be accessed, so set it to a null pointer
                let vtable_ptr = value.get(0).map_or(
                    null::<()>(),
                    |example| {
                        transmute(metadata(example as &$dyn_type))
                    }
                );

                $crate::DynSliceMut::with_vtable_ptr(value, vtable_ptr)
            }
        }
    };
}
