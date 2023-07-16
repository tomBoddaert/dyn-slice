// Enable the required features (nightly must be used)
#![feature(ptr_metadata, pointer_byte_offsets)]

// Remember to import the `DynSliceMethods` trait to use the methods!
use dyn_slice::{declare_dyn_slice, DynSliceMethods};
use std::fmt::Display;

// Declare and import the `&dyn [Display]` type
// (or you can use `dyn_slice::standard::DisplayDynSlice`)
declare_dyn_slice!(Display, display_dyn_slice);
use display_dyn_slice::DynSlice;

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    // Create the first dyn slice
    let dyn_slice = DynSlice::new(&array);

    let array2: [i16; 3] = [5, 6, 7];

    // Create the second dyn slice
    let dyn_slice2 = DynSlice::new(&array2);

    // The iterators can be chained because they are iterators
    // over `&dyn Display` rather than over the underlying types
    let iter = dyn_slice.iter().chain(dyn_slice2.iter());
    for n in iter {
        println!("{n}");
    }
}
