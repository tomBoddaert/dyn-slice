// A dyn slice of any trait that requires Debug can be debug printed.
// For example, DynSlice<dyn Error> or DynSlice<dyn Dyn>.

// Enable the required features (nightly must be used)
#![feature(ptr_metadata, pointer_byte_offsets)]

use dyn_slice::standard::debug;

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    // Create the first dyn slice
    let slice = debug::new(&array);

    // Debug print the slice
    println!("{slice:?}");
}

// Test the example (this can be ignored)
#[test]
fn test() {
    main()
}
