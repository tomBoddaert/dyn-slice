#![feature(ptr_metadata, pointer_byte_offsets)]

use std::fmt::Display;

use dyn_slice::{declare_dyn_slice, DynSliceTrait};

declare_dyn_slice!(Display, display_dyn_slice);
use display_dyn_slice::*;

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    let dyn_slice = DynSlice::new(&array);

    let array2: [i16; 3] = [5, 6, 7];

    let dyn_slice2 = DynSlice::new(&array2);

    let iter = dyn_slice.iter().chain(dyn_slice2.iter());
    for n in iter {
        println!("{n}");
    }
}
