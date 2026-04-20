use core::{
    ops::{Bound, Range},
    ptr::{DynMetadata, NonNull, Pointee},
};

use crate::{DynSlice, DynSliceMut, DynSlicePtr};

// TODO: does this properly protect the traits?
// - What about `unsafe impl SliceIndex<X> for usize` for custom `X`?
trait Sealed {}

#[expect(private_bounds)]
pub unsafe trait DynSliceIndexUnsafe<Dyn>: Sealed
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output;

    unsafe fn get_unchecked(self, slice: DynSlicePtr<Dyn>) -> Self::Output;
}

#[expect(private_bounds)]
pub unsafe trait SliceIndex<T>: Sealed {
    type Output;

    fn get(self, slice: T) -> Option<Self::Output>;
    // TODO: fn index(self, slice: T) -> Self::Output;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StartAndLen {
    start: usize,
    len: usize,
}

impl Sealed for usize {}
unsafe impl<Dyn> DynSliceIndexUnsafe<Dyn> for usize
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = NonNull<Dyn>;

    #[inline]
    unsafe fn get_unchecked(self, slice: DynSlicePtr<Dyn>) -> Self::Output {
        debug_assert!(self <= slice.len);
        let ptr = unsafe { slice.add(self) };
        NonNull::from_raw_parts(ptr, slice.dyn_metadata)
    }
}
unsafe impl<'slice, Dyn> SliceIndex<DynSlice<'slice, Dyn>> for usize
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = &'slice Dyn;

    fn get(self, slice: DynSlice<'slice, Dyn>) -> Option<Self::Output> {
        if slice.ptr.len <= self {
            return None;
        }

        let ptr = unsafe { self.get_unchecked(slice.ptr) };
        Some(unsafe { ptr.as_ref() })
    }
}
unsafe impl<'slice, Dyn> SliceIndex<DynSliceMut<'slice, Dyn>> for usize
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = &'slice mut Dyn;

    fn get(self, slice: DynSliceMut<'slice, Dyn>) -> Option<Self::Output> {
        if slice.ptr.len <= self {
            return None;
        }

        let mut ptr = unsafe { self.get_unchecked(slice.ptr) };
        Some(unsafe { ptr.as_mut() })
    }
}

impl Sealed for StartAndLen {}
unsafe impl<Dyn> DynSliceIndexUnsafe<Dyn> for StartAndLen
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSlicePtr<Dyn>;

    #[inline]
    unsafe fn get_unchecked(self, slice: DynSlicePtr<Dyn>) -> Self::Output {
        debug_assert!(self.start + self.len <= slice.len);
        DynSlicePtr {
            addr: unsafe { slice.add(self.start) },
            len: self.len,
            dyn_metadata: slice.dyn_metadata,
        }
    }
}
// TODO: if StartAndLen made public, implement SliceIndex for StartAndLen

impl Sealed for Range<usize> {}
// TODO: if DynPtrSlice made public, implement SlideIndexUnchecked for Range
unsafe impl<'slice, Dyn> SliceIndex<DynSlice<'slice, Dyn>> for Range<usize>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSlice<'slice, Dyn>;

    fn get(self, slice: DynSlice<'slice, Dyn>) -> Option<Self::Output> {
        if let Some(new_len) = usize::checked_sub(self.end, self.start)
            && self.end <= slice.len()
        {
            let start_len = StartAndLen {
                start: self.start,
                len: new_len,
            };
            let ptr = unsafe { start_len.get_unchecked(slice.ptr) };
            Some(unsafe { ptr.as_ref() })
        } else {
            None
        }
    }
}
unsafe impl<'slice, Dyn> SliceIndex<DynSliceMut<'slice, Dyn>> for Range<usize>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSliceMut<'slice, Dyn>;

    fn get(self, slice: DynSliceMut<'slice, Dyn>) -> Option<Self::Output> {
        if let Some(new_len) = usize::checked_sub(self.end, self.start)
            && self.end <= slice.len()
        {
            let start_len = StartAndLen {
                start: self.start,
                len: new_len,
            };
            let ptr = unsafe { start_len.get_unchecked(slice.ptr) };
            Some(unsafe { ptr.as_mut() })
        } else {
            None
        }
    }
}

// Taken from Rust std library
// MIT OR Apache 2.0 licensed
// https://github.com/rust-lang/rust/blob/b2f1ccf524a3a4cf9c34545167cc23b659cf1cbd/library/core/src/slice/index.rs#L979-L1006
const fn try_bounds_into_range(
    (start, end): (Bound<usize>, Bound<usize>),
    len: usize,
) -> Option<Range<usize>> {
    let end = match end {
        Bound::Included(end) if end >= len => return None,
        Bound::Included(end) => end + 1,

        Bound::Excluded(end) if end > len => return None,
        Bound::Excluded(end) => end,

        Bound::Unbounded => len,
    };

    let start = match start {
        Bound::Excluded(start) if start >= end => return None,
        Bound::Excluded(start) => start + 1,

        Bound::Included(start) if start > end => return None,
        Bound::Included(start) => start,

        Bound::Unbounded => 0,
    };

    Some(start..end)
}
fn try_bounds_into_start_len(
    bounds: (Bound<usize>, Bound<usize>),
    len: usize,
) -> Option<StartAndLen> {
    try_bounds_into_range(bounds, len).map(|range| StartAndLen {
        start: range.start,
        len: unsafe { range.end.unchecked_sub(range.start) },
    })
}

impl Sealed for (Bound<usize>, Bound<usize>) {}
unsafe impl<'slice, Dyn> SliceIndex<DynSlice<'slice, Dyn>> for (Bound<usize>, Bound<usize>)
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSlice<'slice, Dyn>;

    fn get(self, slice: DynSlice<'slice, Dyn>) -> Option<Self::Output> {
        try_bounds_into_start_len(self, slice.ptr.len).map(|start_len| {
            let ptr = unsafe { start_len.get_unchecked(slice.ptr) };
            unsafe { ptr.as_ref() }
        })
    }
}
unsafe impl<'slice, Dyn> SliceIndex<DynSliceMut<'slice, Dyn>> for (Bound<usize>, Bound<usize>)
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSliceMut<'slice, Dyn>;

    fn get(self, slice: DynSliceMut<'slice, Dyn>) -> Option<Self::Output> {
        try_bounds_into_start_len(self, slice.ptr.len).map(|start_len| {
            let ptr = unsafe { start_len.get_unchecked(slice.ptr) };
            unsafe { ptr.as_mut() }
        })
    }
}
