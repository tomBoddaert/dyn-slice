use core::{
    any::Any,
    borrow::Borrow,
    cmp::{PartialEq, PartialOrd},
    fmt::{Debug, Display},
};

use super::declare_dyn_slice;

declare_dyn_slice!(Any, any_dyn_slice);
pub use any_dyn_slice::DynSlice as AnyDynSlice;

declare_dyn_slice!(<T>, AsRef:<T>, as_ref_dyn_slice);
pub use as_ref_dyn_slice::DynSlice as AsRefDynSlice;

declare_dyn_slice!(<T>, Borrow:<T>, borrow_dyn_slice);
pub use borrow_dyn_slice::DynSlice as BorrowDynSlice;

declare_dyn_slice!(<Rhs>, PartialEq:<Rhs>, partial_eq_dyn_slice);
pub use partial_eq_dyn_slice::DynSlice as PartialEqDynSlice;

declare_dyn_slice!(<Rhs>, PartialOrd:<Rhs>, partial_ord_dyn_slice);
pub use partial_ord_dyn_slice::DynSlice as PartialOrdDynSlice;

declare_dyn_slice!(Debug, debug_dyn_slice);
pub use debug_dyn_slice::DynSlice as DebugDynSlice;

declare_dyn_slice!(Display, display_dyn_slice);
pub use display_dyn_slice::DynSlice as DisplayDynSlice;

pub trait To<T> {
    fn to(&self) -> T;
}

impl<T: From<F>, F: Clone> To<T> for F {
    fn to(&self) -> T {
        T::from(self.clone())
    }
}

declare_dyn_slice!(<T>, To:<T>, to_dyn_slice);
pub use to_dyn_slice::DynSlice as ToDynSlice;

#[cfg(feature = "alloc")]
mod alloc_lib {
    extern crate alloc;

    // Currently unused
    // Remove the next 2 lines when in use
    #[allow(unused_imports)]
    use alloc::boxed;
}
#[cfg(feature = "alloc")]
pub use alloc_lib::*;

#[cfg(feature = "std")]
mod std_lib {
    use std::{error::Error, string::ToString};

    use crate::declare_dyn_slice;

    declare_dyn_slice!(Error, error_dyn_slice);
    /// (Only avaliable with the `std` feature)  
    pub use error_dyn_slice::DynSlice as ErrorDynSlice;

    declare_dyn_slice!(ToString, to_string_dyn_slice);
    /// (Only avaliable with the `std` feature)  
    pub use to_string_dyn_slice::DynSlice as ToStringDynSlice;
}
#[cfg(feature = "std")]
pub use std_lib::*;

#[cfg(test)]
mod test {
    use super::To;

    #[test]
    fn test_to() {
        let a: u8 = 5;
        let b: u8 = <u8 as To<u8>>::to(&a);

        assert_eq!(a, b);
    }
}
