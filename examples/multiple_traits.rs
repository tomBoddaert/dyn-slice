// Enable the required features (nightly must be used)
#![feature(ptr_metadata)]

use dyn_slice::declare_new_fns;
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

// Declare the new functions
declare_new_fns!(
    debug_display_slice DebugDisplay
);

fn main() {
    let array: [Wrapper<u8>; 4] = [Wrapper(1), Wrapper(2), Wrapper(3), Wrapper(4)];

    // Create the first dyn slice
    let slice = debug_display_slice::new(&array);

    let array2: [Wrapper<i16>; 3] = [Wrapper(5), Wrapper(6), Wrapper(7)];

    // Create the second dyn slice
    let slice2 = debug_display_slice::new(&array2);

    // The iterators can be chained because they are iterators
    // over `&dyn DebugDisplay` rather than over the underlying types
    let iter = slice.iter().chain(slice2.iter());
    println!("Debug:");
    for n in iter {
        println!("{n:?}");
    }

    let iter = slice.iter().chain(slice2.iter());
    println!("\nDisplay:");
    for n in iter {
        println!("{n}");
    }
}

// Test the example (this can be ignored)
#[test]
fn test() {
    main()
}
