use core::{
    ops::{Bound, Range},
    ptr::{DynMetadata, NonNull, Pointee},
};

use crate::{DynSlice, DynSliceMut, DynSlicePtr};

// TODO: does this properly protect the traits?
// - What about `unsafe impl SliceIndex<X> for usize` for custom `X`?
trait Sealed {}

#[expect(private_bounds)]
pub unsafe trait PtrIndex<Dyn>: Sealed
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
    fn index(self, slice: T) -> Self::Output;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StartAndLen {
    pub start: usize,
    pub len: usize,
}

impl Sealed for usize {}
unsafe impl<Dyn> PtrIndex<Dyn> for usize
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

    #[inline]
    fn get(self, slice: DynSlice<'slice, Dyn>) -> Option<Self::Output> {
        if slice.ptr.len <= self {
            return None;
        }

        let ptr = unsafe { self.get_unchecked(slice.ptr) };
        Some(unsafe { ptr.as_ref() })
    }

    #[inline]
    fn index(self, slice: DynSlice<'slice, Dyn>) -> Self::Output {
        assert!(
            slice.ptr.len > self,
            "index out of bounds: the len is {} but the index is {self}",
            slice.len()
        );

        let ptr = unsafe { self.get_unchecked(slice.ptr) };
        unsafe { ptr.as_ref() }
    }
}
unsafe impl<'slice, Dyn> SliceIndex<DynSliceMut<'slice, Dyn>> for usize
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = &'slice mut Dyn;

    #[inline]
    fn get(self, slice: DynSliceMut<'slice, Dyn>) -> Option<Self::Output> {
        if slice.ptr.len <= self {
            return None;
        }

        let mut ptr = unsafe { self.get_unchecked(slice.ptr) };
        Some(unsafe { ptr.as_mut() })
    }

    #[inline]
    fn index(self, slice: DynSliceMut<'slice, Dyn>) -> Self::Output {
        assert!(
            slice.ptr.len > self,
            "index out of bounds: the len is {} but the index is {self}",
            slice.len()
        );

        let mut ptr = unsafe { self.get_unchecked(slice.ptr) };
        unsafe { ptr.as_mut() }
    }
}

impl Sealed for StartAndLen {}
unsafe impl<Dyn> PtrIndex<Dyn> for StartAndLen
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
unsafe impl<'slice, Dyn> SliceIndex<DynSlice<'slice, Dyn>> for StartAndLen
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSlice<'slice, Dyn>;

    #[inline]
    fn get(self, slice: DynSlice<'slice, Dyn>) -> Option<Self::Output> {
        if let Some(end) = self.start.checked_add(self.len)
            && end <= slice.ptr.len
        {
            let ptr = unsafe { self.get_unchecked(slice.ptr) };
            Some(unsafe { ptr.as_ref() })
        } else {
            None
        }
    }

    #[inline]
    fn index(self, slice: DynSlice<'slice, Dyn>) -> Self::Output {
        if let Some(end) = self.start.checked_add(self.len)
            && end <= slice.ptr.len
        {
            let ptr = unsafe { self.get_unchecked(slice.ptr) };
            unsafe { ptr.as_ref() }
        } else {
            panic!(
                "index range out of bounds: the len is {} but the end is {} + {}",
                slice.ptr.len, self.start, self.len
            );
        }
    }
}
unsafe impl<'slice, Dyn> SliceIndex<DynSliceMut<'slice, Dyn>> for StartAndLen
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSliceMut<'slice, Dyn>;

    #[inline]
    fn get(self, slice: DynSliceMut<'slice, Dyn>) -> Option<Self::Output> {
        if let Some(end) = self.start.checked_add(self.len)
            && end <= slice.ptr.len
        {
            let ptr = unsafe { self.get_unchecked(slice.ptr) };
            Some(unsafe { ptr.as_mut() })
        } else {
            None
        }
    }

    #[inline]
    fn index(self, slice: DynSliceMut<'slice, Dyn>) -> Self::Output {
        if let Some(end) = self.start.checked_add(self.len)
            && end <= slice.ptr.len
        {
            let ptr = unsafe { self.get_unchecked(slice.ptr) };
            unsafe { ptr.as_mut() }
        } else {
            panic!(
                "index out of bounds: the len is {} but the end is {} + {}",
                slice.ptr.len, self.start, self.len
            );
        }
    }
}

impl Sealed for Range<usize> {}
unsafe impl<Dyn> PtrIndex<Dyn> for Range<usize>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSlicePtr<Dyn>;

    #[inline]
    unsafe fn get_unchecked(self, slice: DynSlicePtr<Dyn>) -> Self::Output {
        debug_assert!(self.end >= self.start);
        debug_assert!(slice.len >= self.end);
        let new_len = unsafe { self.end.unchecked_sub(self.start) };
        DynSlicePtr {
            addr: unsafe { slice.add(self.start) },
            len: new_len,
            dyn_metadata: slice.dyn_metadata,
        }
    }
}
unsafe impl<'slice, Dyn> SliceIndex<DynSlice<'slice, Dyn>> for Range<usize>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSlice<'slice, Dyn>;

    #[inline]
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

    #[inline]
    fn index(self, slice: DynSlice<'slice, Dyn>) -> Self::Output {
        assert!(
            self.end <= slice.ptr.len,
            "index range out of bounds: the len is {} but the end is {}",
            slice.ptr.len,
            self.end
        );
        assert!(
            self.start <= self.end,
            "index range start ({}) is larger than end ({})",
            self.start,
            self.end
        );

        let ptr = unsafe { self.get_unchecked(slice.ptr) };
        unsafe { ptr.as_ref() }
    }
}
unsafe impl<'slice, Dyn> SliceIndex<DynSliceMut<'slice, Dyn>> for Range<usize>
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSliceMut<'slice, Dyn>;

    #[inline]
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

    #[inline]
    fn index(self, slice: DynSliceMut<'slice, Dyn>) -> Self::Output {
        assert!(
            self.end <= slice.ptr.len,
            "index range out of bounds: the len is {} but the end is {}",
            slice.ptr.len,
            self.end
        );
        assert!(
            self.start <= self.end,
            "index range start ({}) is larger than end ({})",
            self.start,
            self.end
        );

        let ptr = unsafe { self.get_unchecked(slice.ptr) };
        unsafe { ptr.as_mut() }
    }
}

// Taken from Rust std library
// MIT OR Apache 2.0 licensed
// https://github.com/rust-lang/rust/blob/b2f1ccf524a3a4cf9c34545167cc23b659cf1cbd/library/core/src/slice/index.rs#L979-L1006
#[inline]
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
#[inline]
fn try_bounds_into_start_len(
    bounds: (Bound<usize>, Bound<usize>),
    len: usize,
) -> Option<StartAndLen> {
    try_bounds_into_range(bounds, len).map(|range| StartAndLen {
        start: range.start,
        len: unsafe { range.end.unchecked_sub(range.start) },
    })
}
#[inline]
fn bounds_into_range((start, end): (Bound<usize>, Bound<usize>), len: usize) -> Range<usize> {
    let end = match end {
        Bound::Included(end) if end >= len => {
            panic!("index range out of bounds: the len is {len} but the last element is {end}");
        }
        Bound::Included(end) => end + 1,

        Bound::Excluded(end) if end > len => panic!(),
        Bound::Excluded(end) => end,

        Bound::Unbounded => len,
    };

    let start = match start {
        Bound::Excluded(start) if start >= end => {
            panic!("index range exclusive start ({start}) is larger than or equal to end ({end})");
        }
        Bound::Excluded(start) => start + 1,

        Bound::Included(start) if start > end => {
            panic!("index range start ({start}) is larger than end ({end})")
        }
        Bound::Included(start) => start,

        Bound::Unbounded => 0,
    };

    start..end
}
#[inline]
fn bounds_into_start_len(bounds: (Bound<usize>, Bound<usize>), len: usize) -> StartAndLen {
    let range = bounds_into_range(bounds, len);
    StartAndLen {
        start: range.start,
        len: unsafe { range.end.unchecked_sub(range.start) },
    }
}

impl Sealed for (Bound<usize>, Bound<usize>) {}
unsafe impl<Dyn> PtrIndex<Dyn> for (Bound<usize>, Bound<usize>)
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSlicePtr<Dyn>;

    unsafe fn get_unchecked(self, slice: DynSlicePtr<Dyn>) -> Self::Output {
        let start = match self.0 {
            Bound::Included(start) => start,
            Bound::Excluded(start) => {
                debug_assert!(start.checked_add(1).is_some());
                unsafe { start.unchecked_add(1) }
            }
            Bound::Unbounded => 0,
        };
        debug_assert!(start <= slice.len);

        let len = match self.1 {
            Bound::Included(end) => {
                debug_assert!(end >= start);
                debug_assert!(end < slice.len);
                let last = unsafe { end.unchecked_sub(start) };
                unsafe { last.unchecked_add(1) }
            }
            Bound::Excluded(end) => {
                debug_assert!(end >= start);
                debug_assert!(end <= slice.len);
                unsafe { end.unchecked_sub(start) }
            }
            Bound::Unbounded => unsafe { slice.len.unchecked_sub(start) },
        };

        unsafe { StartAndLen { start, len }.get_unchecked(slice) }
    }
}
unsafe impl<'slice, Dyn> SliceIndex<DynSlice<'slice, Dyn>> for (Bound<usize>, Bound<usize>)
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSlice<'slice, Dyn>;

    #[inline]
    fn get(self, slice: DynSlice<'slice, Dyn>) -> Option<Self::Output> {
        try_bounds_into_start_len(self, slice.ptr.len).map(|start_len| {
            let ptr = unsafe { start_len.get_unchecked(slice.ptr) };
            unsafe { ptr.as_ref() }
        })
    }

    #[inline]
    fn index(self, slice: DynSlice<'slice, Dyn>) -> Self::Output {
        let start_len = bounds_into_start_len(self, slice.ptr.len);
        let ptr = unsafe { start_len.get_unchecked(slice.ptr) };
        unsafe { ptr.as_ref() }
    }
}
unsafe impl<'slice, Dyn> SliceIndex<DynSliceMut<'slice, Dyn>> for (Bound<usize>, Bound<usize>)
where
    Dyn: ?Sized + Pointee<Metadata = DynMetadata<Dyn>>,
{
    type Output = DynSliceMut<'slice, Dyn>;

    #[inline]
    fn get(self, slice: DynSliceMut<'slice, Dyn>) -> Option<Self::Output> {
        try_bounds_into_start_len(self, slice.ptr.len).map(|start_len| {
            let ptr = unsafe { start_len.get_unchecked(slice.ptr) };
            unsafe { ptr.as_mut() }
        })
    }

    #[inline]
    fn index(self, slice: DynSliceMut<'slice, Dyn>) -> Self::Output {
        let start_len = bounds_into_start_len(self, slice.ptr.len);
        let ptr = unsafe { start_len.get_unchecked(slice.ptr) };
        unsafe { ptr.as_mut() }
    }
}
