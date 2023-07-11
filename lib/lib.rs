#![feature(ptr_metadata, pointer_byte_offsets)]
#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::perf,
    clippy::cargo
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(debug_assertions, not(feature = "std")))]
#[doc(hidden)]
mod no_std_test;

#[macro_export]
macro_rules! declare_dyn_slice {
    ( $(< $( $gen:ident ),* >,)? $tr:path$( :<$( $trgen:ident ),*> )?, $vis:vis $name:ident $(,)? ) => {
        $vis mod $name {
            use super::*;
            use $tr as Trait;

            #[derive(Clone, Copy)]
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

                #[allow(dead_code)]
                #[must_use]
                pub const fn len(&self) -> usize {
                    self.len
                }

                #[allow(dead_code)]
                #[must_use]
                pub const fn is_empty(&self) -> bool {
                    self.len == 0
                }

                #[allow(dead_code)]
                #[must_use]
                pub fn first(&self) -> Option<&dyn Trait $(< $($trgen),* >)?> {
                    if self.is_empty() {
                        None
                    } else {
                        Some(unsafe {
                            core::ptr::from_raw_parts::<dyn Trait $(< $($trgen),* >)?>(
                                self.data,
                                self.metadata
                            ).as_ref().unwrap()
                        })
                    }
                }

                #[allow(dead_code)]
                #[must_use]
                pub fn last(&self) -> Option<&dyn Trait $(< $($trgen),* >)?> {
                    if self.is_empty() {
                        None
                    } else {
                        Some(unsafe { self.get_unchecked(self.len - 1) })
                    }
                }

                #[allow(dead_code)]
                #[must_use]
                pub fn get(&self, index: usize) -> Option<&dyn Trait $(< $($trgen),* >)?> {
                    if index >= self.len {
                        None
                    } else {
                        Some(unsafe { self.get_unchecked(index) })
                    }
                }

                #[allow(dead_code)]
                #[must_use]
                /// # Safety
                /// The caller must ensure that index < self.len()
                /// Calling this on an empty dyn Slice will result in a segfault!
                pub unsafe fn get_unchecked(&'a self, index: usize) -> &'a dyn Trait $(< $($trgen),* >)? {
                    unsafe {
                        core::ptr::from_raw_parts::<dyn Trait $(< $($trgen),* >)?>(
                            self.data.byte_add(self.metadata.size_of() * index),
                            self.metadata
                        ).as_ref().unwrap()
                    }
                }
            }

            impl<'a $($(, $gen )*)?> core::ops::Index<usize> for DynSlice<'a $($(, $gen )*)?>
            where
                Self: 'a
            {
                type Output = dyn Trait $(< $($trgen),* >)?;

                fn index(&self, index: usize) -> &Self::Output {
                    use core::mem::transmute;

                    if index >= self.len {
                        panic!("index out of bounds");
                    }

                    // Safety:
                    // The source slice is guaranteed to live for 'a, so the lifetime can
                    // be extended to equal the lifetime of &self, as it must be at least 'a
                    unsafe { transmute(self.get_unchecked(index)) }
                }
            }

            impl<'a $($(, $gen )*)?> DynSlice<'a $($(, $gen )*)?> {
                #[allow(dead_code)]
                #[must_use]
                pub const fn iter(&'a self) -> Iter<'a $($(, $gen )*)?> {
                    Iter {
                        slice: self,
                        next_index: 0,
                    }
                }
            }

            #[derive(Clone)]
            pub struct Iter<'a $($(, $gen )*)?> {
                slice: &'a DynSlice<'a $($(, $gen )*)?>,
                next_index: usize,
            }

            impl<'a $($(, $gen )*)?> Iterator for Iter<'a $($(, $gen )*)?> {
                type Item = &'a dyn Trait $(< $($trgen),* >)?;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.next_index == self.slice.len {
                        None
                    } else {
                        let element = unsafe { self.slice.get_unchecked(self.next_index) };
                        self.next_index += 1;

                        Some(element)
                    }
                }

                fn size_hint(&self) -> (usize, Option<usize>) {
                    let remaining = self.slice.len - self.next_index;
                    (remaining, Some(remaining))
                }

                fn count(self) -> usize {
                    self.slice.len - self.next_index
                }

                fn nth(&mut self, n: usize) -> Option<Self::Item> {
                    let index = self.next_index + n;
                    if index >= self.slice.len {
                        self.next_index = self.slice.len;
                        return None;
                    }

                    self.next_index = index;
                    self.next()
                }

                fn last(self) -> Option<Self::Item> {
                    if self.next_index == self.slice.len {
                        None
                    } else {
                        self.slice.last()
                    }
                }
            }

            impl<'a $($(, $gen )*)?> core::iter::FusedIterator for Iter<'a $($(, $gen )*)?> {}
            impl<'a $($(, $gen )*)?> ExactSizeIterator for Iter<'a $($(, $gen )*)?> {}
        }
    };
}

#[cfg(test)]
mod test {
    use std::fmt::Display;

    use super::declare_dyn_slice;

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
