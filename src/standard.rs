use core::{
    any::{Any, TypeId},
    fmt,
    ptr::{DynMetadata, NonNull, Pointee},
};

use crate::{DynSlice, DynSliceMut};

macro_rules! impl_any {
    ( $any:ty ) => {
        impl<'slice> DynSlice<'slice, $any> {
            pub fn element_type_id(self) -> Option<TypeId> {
                self.first().map(|first| first.type_id())
            }

            pub fn element_is<T>(self) -> bool
            where
                T: Any,
            {
                self.first().is_some_and(|first| first.is::<T>())
            }

            pub fn downcast_slice<T>(self) -> Option<&'slice [T]>
            where
                T: Any,
            {
                self.element_is::<T>().then(|| {
                    let ptr = NonNull::from_raw_parts(self.ptr.addr.cast::<T>(), self.ptr.len);
                    unsafe { ptr.as_ref() }
                })
            }
        }

        impl<'slice> DynSliceMut<'slice, $any> {
            pub fn downcast_slice_mut<T>(&mut self) -> Option<&mut [T]>
            where
                T: Any,
            {
                self.coerce_shared().element_is::<T>().then(|| {
                    let mut ptr = NonNull::from_raw_parts(self.ptr.addr.cast::<T>(), self.ptr.len);
                    unsafe { ptr.as_mut() }
                })
            }
        }
    };
}

impl_any!(dyn Any);
impl_any!(dyn Any + Send);
impl_any!(dyn Any + Send + Sync);

impl<Dyn, Rhs> PartialEq<[Rhs]> for DynSlice<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + PartialEq<Rhs>,
{
    fn eq(&self, other: &[Rhs]) -> bool {
        // TODO: if we can trigger the short-circuit in `Iterator::eq`,
        // then we don't need to check for length equality
        // - `TrustedLen` is implemented for both iterators
        self.len() == other.len() && self.iter().eq(other)
    }
}
impl<Dyn, Rhs> PartialOrd<[Rhs]> for DynSlice<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + PartialOrd<Rhs>,
{
    fn partial_cmp(&self, other: &[Rhs]) -> Option<core::cmp::Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<Dyn> fmt::Debug for DynSlice<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}
impl<Dyn> fmt::Debug for DynSliceMut<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>> + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.coerce_shared()).finish()
    }
}
