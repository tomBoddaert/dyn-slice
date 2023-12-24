// Enable the required features (nightly must be used)
#![feature(ptr_metadata)]

use dyn_slice::declare_new_fns;
use std::fmt::Display;

// Declare the `new` function
// (or you can use `dyn_slice::standard::display::new`
declare_new_fns!(display_slice Display);

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    // Create the first dyn slice
    let slice = display_slice::new(&array);

    let array2: [i16; 3] = [5, 6, 7];

    // Create the second dyn slice
    let slice2 = display_slice::new(&array2);

    // The iterators can be chained because they are iterators
    // over `&dyn Display` rather than over the underlying types
    let iter = slice.iter().chain(slice2.iter());
    for n in iter {
        println!("{n}");
    }
}

// Test the example (this can be ignored)
#[test]
fn test() {
    main()
}
