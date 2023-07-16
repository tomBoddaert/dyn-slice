// Enable the required features (nightly must be used)
#![feature(ptr_metadata, pointer_byte_offsets)]

// Remember to import the `DynSliceMethods` trait to use the methods!
use dyn_slice::{declare_dyn_slice, DynSliceMethods};
use std::fmt::{Debug, Display};

// If up to one trait is not auto-implemented, you can use the trait_alias feature
// example:
//   trait $New = $Trait + $auto;

// Create our trait that combines the Debug and Display traits
pub trait DebugDisplay: Debug + Display {}
impl<T: Debug + Display> DebugDisplay for T {}

// Create a wrapper that can be seen when printed in debug mode
#[derive(Clone, Copy, Debug)]
struct Wrapper<T>(T);

impl<T: Display> Display for Wrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

// Declare and import the `&dyn [DebugDisplay]` type
declare_dyn_slice!(DebugDisplay, debug_display_dyn_slice);
use debug_display_dyn_slice::*;

fn main() {
    let array: [Wrapper<u8>; 4] = [Wrapper(1), Wrapper(2), Wrapper(3), Wrapper(4)];

    // Create the first dyn slice
    let dyn_slice = DynSlice::new(&array);

    let array2: [Wrapper<i16>; 3] = [Wrapper(5), Wrapper(6), Wrapper(7)];

    // Create the second dyn slice
    let dyn_slice2 = DynSlice::new(&array2);

    // The iterators can be chained because they are iterators
    // over `&dyn DebugDisplay` rather than over the underlying types
    let iter = dyn_slice.iter().chain(dyn_slice2.iter());
    println!("Debug:");
    for n in iter {
        println!("{n:?}");
    }

    let iter = dyn_slice.iter().chain(dyn_slice2.iter());
    println!("\nDisplay:");
    for n in iter {
        println!("{n}");
    }
}
