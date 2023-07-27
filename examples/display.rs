// Enable the required features (nightly must be used)
#![feature(ptr_metadata, pointer_byte_offsets)]

use dyn_slice::declare_new_fn;
use std::fmt::Display;

// Declare and import the `new` function
// (or you can use `dyn_slice::standard::display::new`
declare_new_fn!(Display, display_dyn_slice);
use display_dyn_slice::new as new_dyn_slice;

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    // Create the first dyn slice
    let dyn_slice = new_dyn_slice(&array);

    let array2: [i16; 3] = [5, 6, 7];

    // Create the second dyn slice
    let dyn_slice2 = new_dyn_slice(&array2);

    // The iterators can be chained because they are iterators
    // over `&dyn Display` rather than over the underlying types
    let iter = dyn_slice.iter().chain(dyn_slice2.iter());
    for n in iter {
        println!("{n}");
    }
}

// Test the example (this can be ignored)
#[test]
fn test() {
    main()
}
