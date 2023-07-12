#![feature(ptr_metadata, pointer_byte_offsets)]
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::perf,
    clippy::cargo
)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "alloc", feature(allocator_api))]

mod dyn_slice_trait;
pub use dyn_slice_trait::*;
pub mod standard;

#[cfg(all(debug_assertions, not(feature = "std")))]
#[doc(hidden)]
mod no_std_test;

#[macro_export]
/// Declares a new dyn slice and implements the [`DynSliceTrait`] trait on it
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
    ( $(< $( $gen:ident ),* >,)? $tr:path$( :<$( $trgen:ident ),*> )?, $vis:vis $name:ident $(,)? ) => {
        $vis mod $name {
            #[allow(unused_imports)]
            use super::*;
            use $tr as Trait;

            #[derive(Clone, Copy)]
            #[doc = concat!("`&dyn [", stringify!($tr), "]`")]
            pub struct DynSlice<'a $($(, $gen )*)?> {
                metadata: core::ptr::DynMetadata<dyn Trait $(< $($trgen),* >)?>,
                len: usize,
                data: *const (),
                phantom: core::marker::PhantomData<&'a ()>,
            }

            impl<'a $($(, $gen )*)?> DynSlice<'a $($(, $gen )*)?> {
                #[allow(dead_code)]
                #[must_use]
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
                        let metadata = value.get(0).map_or(
                            transmute(null::<()>()),
                            |example| {
                                metadata(transmute::<
                                    _,
                                    &'static dyn Trait $(< $($trgen),* >)?
                                >(example as &dyn Trait $(< $($trgen),* >)?))
                            }
                        );

                        Self::with_metadata(value, metadata)
                    }
                }

                #[must_use]
                /// # Safety
                #[doc = concat!("Caller must ensure that `metadata` is a valid instance of `DynMetadata` for `DynSliceFromType` and `", stringify!($tr), "`")]
                pub const unsafe fn with_metadata<DynSliceFromType: Trait $(< $($trgen),* >)?>(
                    value: &'a [DynSliceFromType],
                    metadata: core::ptr::DynMetadata<dyn Trait $(< $($trgen),* >)?>
                ) -> Self {
                    Self {
                        metadata,
                        len: value.len(),
                        data: value.as_ptr().cast(),
                        phantom: core::marker::PhantomData,
                    }
                }
            }

            unsafe impl<'a $($(, $gen )*)?> $crate::DynSliceTrait for DynSlice<'a $($(, $gen )*)?> {
                type Dyn = dyn Trait $(< $($trgen),* >)?;

                #[inline]
                unsafe fn from_parts(metadata: core::ptr::DynMetadata<Self::Dyn>, len: usize, data: *const ()) -> Self {
                    Self {
                        metadata,
                        len,
                        data,
                        phantom: core::marker::PhantomData,
                    }
                }

                #[inline]
                fn metadata(&self) -> core::ptr::DynMetadata<Self::Dyn> {
                    self.metadata
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
                type Output = <Self as $crate::DynSliceTrait>::Dyn;

                fn index(&self, index: usize) -> &Self::Output {
                    if index >= self.len {
                        panic!("index out of bounds");
                    }

                    unsafe { <Self as $crate::DynSliceTrait>::get_unchecked(&self, index) }
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::{declare_dyn_slice, DynSliceTrait};

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
