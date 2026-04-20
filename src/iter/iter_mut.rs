use core::{
    iter::{FusedIterator, TrustedLen},
    marker::PhantomData,
    num::NonZero,
    ptr::{DynMetadata, NonNull, Pointee},
};

use crate::{DynSlice, DynSliceMut, DynSlicePtr};

pub struct IterMut<'slice, Dyn>
where
    Dyn: ?Sized,
{
    ptr: DynSlicePtr<Dyn>,
    _phantom: PhantomData<&'slice mut Dyn>,
}

impl<'slice, Dyn> const IntoIterator for DynSliceMut<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Item = &'slice mut Dyn;
    type IntoIter = IterMut<'slice, Dyn>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }
}

impl<'slice, Dyn> IterMut<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[must_use]
    #[inline]
    pub const fn remaining(&self) -> DynSlice<'_, Dyn> {
        DynSlice {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    pub const fn remaining_mut(&mut self) -> DynSliceMut<'_, Dyn> {
        DynSliceMut {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    pub const fn to_remaining(self) -> DynSliceMut<'slice, Dyn> {
        DynSliceMut {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }
}

impl<'slice, Dyn> Iterator for IterMut<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Item = &'slice mut Dyn;

    fn next(&mut self) -> Option<Self::Item> {
        self.ptr.len = self.ptr.len.checked_sub(1)?;

        let ptr = self.ptr.addr;
        self.ptr.addr = unsafe { self.ptr.add(1) };

        let mut ptr = NonNull::from_raw_parts(ptr, self.ptr.dyn_metadata);

        Some(unsafe { ptr.as_mut() })
    }

    fn advance_by(&mut self, n: usize) -> Result<(), core::num::NonZero<usize>> {
        if n > self.ptr.len {
            let remaining = unsafe { NonZero::new_unchecked(n - self.ptr.len) };

            let len = core::mem::replace(&mut self.ptr.len, 0);
            self.ptr.addr = unsafe { self.ptr.add(len) };

            Err(remaining)
        } else {
            self.ptr.len -= n;
            self.ptr.addr = unsafe { self.ptr.add(n) };

            Ok(())
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Dyn> DoubleEndedIterator for IterMut<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let last = self.ptr.len.checked_sub(1)?;
        self.ptr.len = last;

        let ptr = unsafe { self.ptr.add(last) };
        let mut ptr = NonNull::from_raw_parts(ptr, self.ptr.dyn_metadata);

        Some(unsafe { ptr.as_mut() })
    }

    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        if n > self.ptr.len {
            let remaining = unsafe { NonZero::new_unchecked(n - self.ptr.len) };
            self.ptr.len = 0;

            Err(remaining)
        } else {
            self.ptr.len -= n;

            Ok(())
        }
    }
}

impl<Dyn> ExactSizeIterator for IterMut<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn len(&self) -> usize {
        self.ptr.len
    }
}

impl<Dyn> FusedIterator for IterMut<'_, Dyn> where Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>
{}

unsafe impl<Dyn> TrustedLen for IterMut<'_, Dyn> where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>
{
}
