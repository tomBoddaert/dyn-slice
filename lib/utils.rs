use core::{
    mem::transmute,
    ptr::{DynMetadata, Pointee},
};

use crate::{DynSlice, DynSliceMut};

#[must_use]
#[inline]
/// Extend the lifetime of a [`DynSlice`].
///
/// # Safety
/// The original slice this is created from must be immediatly discarded.
pub unsafe fn extend_lifetime<'to, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>>(
    value: DynSlice<Dyn>,
) -> DynSlice<'to, Dyn> {
    transmute(value)
}

#[must_use]
#[inline]
/// Extend the lifetime of a [`DynSliceMut`].
///
/// # Safety
/// The original slice this is created from must be immediatly discarded.
pub unsafe fn extend_lifetime_mut<'to, Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>>(
    value: DynSliceMut<Dyn>,
) -> DynSliceMut<'to, Dyn> {
    transmute(value)
}
