use core::{
    iter::FusedIterator,
    marker::{Destruct, PhantomData},
    ptr::{DynMetadata, Pointee},
};

use crate::{DynSlice, DynSliceMut, DynSlicePtr};

pub struct SplitMut<'slice, Dyn, P>
where
    Dyn: ?Sized,
{
    ptr: DynSlicePtr<Dyn>,
    predicate: P,
    _phantom: PhantomData<&'slice Dyn>,
}

impl<'slice, Dyn, P> SplitMut<'slice, Dyn, P>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
    P: FnMut(&Dyn) -> bool,
{
    #[must_use]
    #[inline]
    pub(crate) const fn new(slice: DynSliceMut<'slice, Dyn>, predicate: P) -> Self {
        SplitMut {
            ptr: slice.ptr,
            predicate,
            _phantom: PhantomData,
        }
    }
}

impl<'slice, Dyn, P> SplitMut<'slice, Dyn, P>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
    P: FnMut(&Dyn) -> bool,
{
    #[must_use]
    #[inline]
    pub const fn remaining(&self) -> DynSlice<'slice, Dyn> {
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
    pub const fn to_remaining(self) -> DynSliceMut<'slice, Dyn>
    where
        P: [const] Destruct,
    {
        DynSliceMut {
            ptr: self.ptr,
            _phantom: PhantomData,
        }
    }
}

impl<'slice, Dyn, P> Iterator for SplitMut<'slice, Dyn, P>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
    P: FnMut(&Dyn) -> bool,
{
    type Item = DynSliceMut<'slice, Dyn>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.len == 0 {
            return None;
        }

        let mut slice = unsafe { self.ptr.as_mut() };

        if let Some(position) = slice
            .coerce_shared()
            .into_iter()
            .position(&mut self.predicate)
        {
            let offset = unsafe { position.unchecked_add(1) };
            self.ptr.len = unsafe { self.ptr.len.unchecked_sub(offset) };
            self.ptr.addr = unsafe { self.ptr.add(offset) };

            slice.ptr.len = position;
        } else {
            let len = self.ptr.len;
            self.ptr.len = 0;
            // Not necessary but makes returned pointers more consistent
            self.ptr.addr = unsafe { self.ptr.add(len) };
        }

        Some(slice)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.ptr.len == 0 {
            (0, Some(0))
        } else {
            (1, Some(self.ptr.len + 1))
        }
    }
}

impl<Dyn, P> DoubleEndedIterator for SplitMut<'_, Dyn, P>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
    P: FnMut(&Dyn) -> bool,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr.len == 0 {
            return None;
        }

        let mut slice = unsafe { self.ptr.as_mut() };

        if let Some(position) = slice
            .coerce_shared()
            .into_iter()
            .rposition(&mut self.predicate)
        {
            let offset = unsafe { position.unchecked_add(1) };
            slice.ptr.len = unsafe { self.ptr.len.unchecked_sub(offset) };
            slice.ptr.addr = unsafe { self.ptr.add(offset) };

            self.ptr.len = position;
        } else {
            self.ptr.len = 0;
        }

        Some(slice)
    }
}

impl<Dyn, P> FusedIterator for SplitMut<'_, Dyn, P>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
    P: FnMut(&Dyn) -> bool,
{
}

impl<Dyn, P> const Clone for SplitMut<'_, Dyn, P>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
    P: FnMut(&Dyn) -> bool + [const] Clone,
{
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            predicate: self.predicate.clone(),
            _phantom: PhantomData,
        }
    }
}
