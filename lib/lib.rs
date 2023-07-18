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
    clippy::cargo
)]
#![cfg_attr(not(feature = "std"), no_std)]

mod dyn_slice_methods;
pub use dyn_slice_methods::*;
/// Dyn slice definitions for some common traits
pub mod standard;

#[macro_export]
/// Declares a new dyn slice and implements the [`DynSliceMethods`] trait on it
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
/// # use dyn_slice::declare_dyn_slice;
/// declare_dyn_slice!(std::fmt::Display, display_dyn_slice);
/// pub use display_dyn_slice::DynSlice as DisplayDynSlice;
/// ```
///
/// Generics:
/// ```
/// pub trait A<B, C> {
///     fn b(&self) -> B;
///     fn c(&self) -> C;
/// }
/// ```
///
/// ```
/// # #![feature(ptr_metadata)]
/// # mod test {
/// # use dyn_slice::declare_dyn_slice;
/// # pub trait A<B, C> {
/// #     fn b(&self) -> B;
/// #     fn c(&self) -> C;
/// # }
/// declare_dyn_slice!(A:<u8, f32>, given_generic_dyn_slice);
/// # }
/// ```
///
/// ```
/// # #![feature(ptr_metadata)]
/// # mod test {
/// # use dyn_slice::declare_dyn_slice;
/// # pub trait A<B, C> {
/// #     fn b(&self) -> B;
/// #     fn c(&self) -> C;
/// # }
/// declare_dyn_slice!(<C>, A:<u8, C>, half_given_generic_dyn_slice);
/// # }
/// ```
///
/// ```
/// # #![feature(ptr_metadata)]
/// # mod test {
/// # use dyn_slice::declare_dyn_slice;
/// # pub trait A<B, C> {
/// #     fn b(&self) -> B;
/// #     fn c(&self) -> C;
/// # }
/// declare_dyn_slice!(<B, C>, A:<B, C>, generic_dyn_slice);
/// # }
/// ```
macro_rules! declare_dyn_slice {
    (
        $(#[ $meta:meta ])*
        $(< $( $gen:ident ),* >,)?
        $tr:path $( :<$( $trgen:ident ),*> )?,
        $vis:vis $name:ident $(,)?
    ) => {
        $vis mod $name {
            #[allow(unused_imports)]
            use super::*;
            use $tr as Trait;

            #[derive(Clone, Copy)]
            #[doc = concat!("`&dyn [", stringify!($tr), "]`")]
            $(#[ $meta ])*
            pub struct DynSlice<'a $($(, $gen )*)?> {
                // metadata: core::ptr::DynMetadata<dyn Trait $(< $($trgen),* >)?>,
                vtable_ptr: *const (),
                len: usize,
                data: *const (),
                phantom: core::marker::PhantomData<&'a dyn Trait $(< $($trgen),* >)?>,
            }

            impl<'a $($(, $gen )*)?> DynSlice<'a $($(, $gen )*)?> {
                #[allow(dead_code)]
                #[must_use]
                /// Create a dyn slice from a slice
                pub fn new<DynSliceFromType: Trait $(< $($trgen),* >)?>(value: &'a [DynSliceFromType]) -> Self
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

                        Self::with_vtable_ptr(value, vtable_ptr)
                    }
                }

                #[must_use]
                /// # Safety
                #[doc = concat!("Caller must ensure that `vtable_ptr` is a valid instance of `DynMetadata` for `DynSliceFromType` and `dyn ", stringify!($tr), "` transmuted, or optionally, a null pointer if `value.len() == 0`.")]
                pub const unsafe fn with_vtable_ptr<DynSliceFromType: Trait $(< $($trgen),* >)?>(
                    value: &'a [DynSliceFromType],
                    vtable_ptr: *const (),
                ) -> Self {
                    Self {
                        vtable_ptr,
                        len: value.len(),
                        data: value.as_ptr().cast(),
                        phantom: core::marker::PhantomData,
                    }
                }

                #[allow(dead_code)]
                #[must_use]
                /// # Safety
                #[doc = concat!("Caller must ensure that `metadata` is a valid instance of `DynMetadata` for `DynSliceFromType` and `dyn ", stringify!($tr), "`")]
                pub const unsafe fn with_metadata<DynSliceFromType: Trait $(< $($trgen),* >)?>(
                    value: &'a [DynSliceFromType],
                    metadata: core::ptr::DynMetadata<dyn Trait $(< $($trgen),* >)?>
                ) -> Self {
                    Self {
                        vtable_ptr: core::mem::transmute(metadata),
                        len: value.len(),
                        data: value.as_ptr().cast(),
                        phantom: core::marker::PhantomData,
                    }
                }
            }

            unsafe impl<'a $($(, $gen )*)?> $crate::DynSliceMethods for DynSlice<'a $($(, $gen )*)?> {
                type Dyn = dyn Trait $(< $($trgen),* >)?;

                #[inline]
                unsafe fn from_parts(vtable_ptr: *const (), len: usize, data: *const ()) -> Self {
                    Self {
                        vtable_ptr,
                        len,
                        data,
                        phantom: core::marker::PhantomData,
                    }
                }

                #[inline]
                fn vtable_ptr(&self) -> *const () {
                    self.vtable_ptr
                }

                #[inline]
                fn len(&self) -> usize {
                    self.len
                }

                #[inline]
                fn as_ptr(&self) -> *const () {
                    self.data
                }
            }

            impl<'a $($(, $gen )*)?> core::ops::Index<usize> for DynSlice<'a $($(, $gen )*)?> {
                type Output = <Self as $crate::DynSliceMethods>::Dyn;

                fn index(&self, index: usize) -> &Self::Output {
                    if index >= self.len {
                        panic!("index out of bounds");
                    }

                    unsafe { <Self as $crate::DynSliceMethods>::get_unchecked(&self, index) }
                }
            }

            impl<'a, DynSliceFromType: Trait $(< $($trgen),* >)? $($(, $gen )*)?> From<&'a [DynSliceFromType]> for DynSlice<'a $($(, $gen )*)?>
            where
                dyn Trait $(< $($trgen),* >)?: core::ptr::Pointee<Metadata = core::ptr::DynMetadata<dyn Trait $(< $($trgen),* >)?>>,
                $($(
                    $gen: 'static
                ),*)?
            {
                fn from(value: &'a [DynSliceFromType]) -> Self {
                    Self::new(value)
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::{declare_dyn_slice, DynSliceMethods};

    declare_dyn_slice!(Display, display_dyn_slice);
    pub use display_dyn_slice::*;

    #[test]
    fn create_dyn_slice() {
        let array: [u8; 5] = [1, 2, 3, 4, 5];

        let dyn_slice = DynSlice::new(&array);

        assert_eq!(dyn_slice.len(), array.len());
        assert!(!dyn_slice.is_empty());

        for (i, x) in array.iter().enumerate() {
            assert_eq!(
                format!(
                    "{}",
                    dyn_slice
                        .get(i)
                        .expect("failed to get an element of dyn_slice")
                ),
                format!("{x}"),
            );
        }
    }

    #[test]
    fn empty() {
        let array: [u8; 0] = [];

        let dyn_slice = DynSlice::new(&array);

        assert_eq!(dyn_slice.len(), 0);
        assert!(dyn_slice.is_empty());
    }
}
