use std::{
    marker::Unsize,
    ptr::{DynMetadata, Pointee},
};

use crate::{DynSlice, DynSliceMut};

pub trait AsDynSlice<'a, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    fn as_dyn_slice(self) -> DynSlice<'a, Dyn>;
}

pub trait AsDynSliceMut<'a, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    fn as_dyn_slice_mut(self) -> DynSliceMut<'a, Dyn>;
}

impl<'slice, Dyn> AsDynSlice<'slice, Dyn> for DynSlice<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn as_dyn_slice(self) -> DynSlice<'slice, Dyn> {
        self
    }
}
impl<'a, 'slice, Dyn> AsDynSlice<'slice, Dyn> for &'a DynSlice<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn as_dyn_slice(self) -> DynSlice<'slice, Dyn> {
        *self
    }
}
impl<'a, 'slice, Dyn> AsDynSlice<'a, Dyn> for &'a DynSliceMut<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn as_dyn_slice(self) -> DynSlice<'a, Dyn> {
        self.coerce_shared()
    }
}

impl<'slice, Dyn> AsDynSliceMut<'slice, Dyn> for DynSliceMut<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn as_dyn_slice_mut(self) -> DynSliceMut<'slice, Dyn> {
        self
    }
}
impl<'a, 'slice, Dyn> AsDynSliceMut<'a, Dyn> for &'a mut DynSliceMut<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn as_dyn_slice_mut(self) -> DynSliceMut<'a, Dyn> {
        self.reborrow()
    }
}

impl<'slice, T, Dyn> AsDynSlice<'slice, Dyn> for &'slice [T]
where
    T: Unsize<Dyn> + 'slice,
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn as_dyn_slice(self) -> DynSlice<'slice, Dyn> {
        DynSlice::new(self)
    }
}
impl<'slice, T, Dyn> AsDynSliceMut<'slice, Dyn> for &'slice mut [T]
where
    T: Unsize<Dyn> + 'slice,
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn as_dyn_slice_mut(self) -> DynSliceMut<'slice, Dyn> {
        DynSliceMut::new(self)
    }
}
