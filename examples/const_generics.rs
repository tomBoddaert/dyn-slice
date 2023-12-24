// Enable the required features (nightly must be used)
#![feature(ptr_metadata)]

use dyn_slice::declare_new_fns;

// Declare a trait with a constant parameter
pub trait AddConst<const N: u8> {
    fn add(&self) -> u8;
}

// Implement the trait for u8
impl<const N: u8> AddConst<N> for u8 {
    fn add(&self) -> u8 {
        self + N
    }
}

// Declare new functions for dyn slices of the trait
declare_new_fns!(
    add_const_slice<const N: u8> AddConst<N>
);

fn main() {
    // Create an array of u8
    let array = [5, 58, 97];
    // Create a dyn slice from the array
    let slice = add_const_slice::new::<12, _>(&array);

    // Add the numbers
    let sums = slice.iter().map(|x| x.add());
    // Print the results
    println!("{:?}", sums.collect::<Vec<u8>>());
}

// Test the example (this can be ignored)
#[test]
fn test() {
    main();
}
