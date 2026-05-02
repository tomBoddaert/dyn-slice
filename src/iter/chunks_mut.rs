use core::{
    iter::{FusedIterator, TrustedLen},
    marker::PhantomData,
    num::NonZero,
    ptr::{DynMetadata, Pointee},
};

use crate::{DynSlice, DynSliceMut, DynSlicePtr};

pub struct ChunksMut<'slice, Dyn>
where
    Dyn: ?Sized,
{
    ptr: DynSlicePtr<Dyn>,
    chunk_size: NonZero<usize>,
    _phantom: PhantomData<&'slice Dyn>,
}

impl<'slice, Dyn> ChunksMut<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[must_use]
    #[inline]
    pub(crate) const fn new(slice: DynSliceMut<'slice, Dyn>, chunk_size: NonZero<usize>) -> Self {
        Self {
            ptr: slice.ptr,
            chunk_size,
            _phantom: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    pub const fn remaining(&self) -> DynSlice<'_, Dyn> {
        unsafe { self.ptr.as_ref() }
    }

    #[must_use]
    #[inline]
    pub const fn remaining_mut(&mut self) -> DynSliceMut<'_, Dyn> {
        unsafe { self.ptr.as_mut() }
    }

    #[must_use]
    #[inline]
    pub const fn to_remaining(self) -> DynSliceMut<'slice, Dyn> {
        unsafe { self.ptr.as_mut() }
    }
}

impl<'slice, Dyn> Iterator for ChunksMut<'slice, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Item = DynSliceMut<'slice, Dyn>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let len = self.chunk_size.min(NonZero::new(self.ptr.len)?).get();

        let ptr = DynSlicePtr {
            addr: self.ptr.addr,
            len,
            dyn_metadata: self.ptr.dyn_metadata,
        };

        self.ptr.len = unsafe { self.ptr.len.unchecked_sub(len) };
        self.ptr.addr = unsafe { self.ptr.add(len) };

        Some(unsafe { ptr.as_mut() })
    }

    #[inline]
    fn advance_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        let chunks = self.len();
        match n.cmp(&chunks) {
            core::cmp::Ordering::Greater => {
                let remaining = unsafe { NonZero::new_unchecked(n - chunks) };

                let len = core::mem::replace(&mut self.ptr.len, 0);
                self.ptr.addr = unsafe { self.ptr.add(len) };

                Err(remaining)
            }

            core::cmp::Ordering::Equal => {
                let len = core::mem::replace(&mut self.ptr.len, 0);
                self.ptr.addr = unsafe { self.ptr.add(len) };
                Ok(())
            }
            core::cmp::Ordering::Less => {
                let offset = unsafe { self.chunk_size.get().unchecked_mul(n) };
                self.ptr.len = unsafe { self.ptr.len.unchecked_sub(offset) };
                self.ptr.addr = unsafe { self.ptr.add(offset) };
                Ok(())
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<Dyn> DoubleEndedIterator for ChunksMut<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.ptr.len == 0 {
            return None;
        }
        let len = NonZero::new(self.ptr.len % self.chunk_size)
            .unwrap_or(self.chunk_size)
            .get();
        let offset = unsafe { self.ptr.len.unchecked_sub(len) };

        let ptr = DynSlicePtr {
            addr: unsafe { self.ptr.addr.add(offset) },
            len,
            dyn_metadata: self.ptr.dyn_metadata,
        };

        self.ptr.len = offset;

        Some(unsafe { ptr.as_mut() })
    }

    #[inline]
    fn advance_back_by(&mut self, n: usize) -> Result<(), NonZero<usize>> {
        let chunks = self.len();
        match n.cmp(&chunks) {
            core::cmp::Ordering::Greater => {
                self.ptr.len = 0;
                Err(unsafe { NonZero::new_unchecked(n - chunks) })
            }

            core::cmp::Ordering::Equal => {
                self.ptr.len = 0;
                Ok(())
            }
            core::cmp::Ordering::Less => {
                // As chunks > n, no need to upper bound by len
                self.ptr.len = unsafe { self.chunk_size.get().unchecked_mul(chunks - n) };
                Ok(())
            }
        }
    }
}

impl<Dyn> ExactSizeIterator for ChunksMut<'_, Dyn>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    #[inline]
    fn len(&self) -> usize {
        self.ptr.len / self.chunk_size
            + usize::from(!self.ptr.len.is_multiple_of(self.chunk_size.get()))
    }
}

unsafe impl<Dyn> TrustedLen for ChunksMut<'_, Dyn> where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>
{
}

impl<Dyn> FusedIterator for ChunksMut<'_, Dyn> where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>
{
}
