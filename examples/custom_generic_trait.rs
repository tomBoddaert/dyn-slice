// Enable the required features (nightly must be used)
#![feature(ptr_metadata, pointer_byte_offsets)]

use dyn_slice::declare_new_fn;

// Create our custom trait with a generic
pub trait MyTrait<T> {
    fn to_t(&self) -> T;
    fn add(&self, rhs: T) -> T;
}

// Implement the trait for u8 with u64
impl MyTrait<u64> for u8 {
    fn to_t(&self) -> u64 {
        u64::from(*self)
    }

    fn add(&self, rhs: u64) -> u64 {
        self.to_t() + rhs
    }
}

// Implement the trait for u16 with u64
impl MyTrait<u64> for u16 {
    fn to_t(&self) -> u64 {
        u64::from(*self)
    }

    fn add(&self, rhs: u64) -> u64 {
        self.to_t() + rhs
    }
}

// Declare and import the `new` function, generic on `T`
declare_new_fn!(<T>, MyTrait:<T>, my_trait_dyn_slice);
use my_trait_dyn_slice::new as new_dyn_slice;

// Declare and import the `new` function
declare_new_fn!(MyTrait:<u64>, my_trait_u64_dyn_slice);
#[allow(unused_imports)]
use my_trait_u64_dyn_slice::new as new_dyn_slice_u64;

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    // Create the first dyn slice
    let dyn_slice = new_dyn_slice::<u64, _>(&array);

    // Get the first and last elements as u64
    let first = dyn_slice.first().map(MyTrait::<u64>::to_t);
    let last = dyn_slice.last().map(MyTrait::<u64>::to_t);
    println!("1: first: {first:?}, last: {last:?}");

    let array2: [u16; 3] = [5, 6, 7];

    // Create the second dyn slice
    let dyn_slice2 = new_dyn_slice::<u64, _>(&array2);

    // Get the first and last elements as u64
    let first = dyn_slice2.first().map(MyTrait::<u64>::to_t);
    let last = dyn_slice2.last().map(MyTrait::<u64>::to_t);

    println!("2: first: {first:?}, last: {last:?}\n");

    // Print the sum of each pair from the dyn slices
    let iter = dyn_slice.iter().zip(dyn_slice2.iter());
    for (i, (a, b)) in iter.enumerate() {
        println!("sum {}: {}", i + 1, a.add(b.to_t()));
    }
}

// Test the example (this can be ignored)
#[test]
fn test() {
    main()
}
