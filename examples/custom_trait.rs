// Enable the required features (nightly must be used)
#![feature(ptr_metadata, pointer_byte_offsets)]

// Remember to import the `DynSliceMethods` trait to use the methods!
use dyn_slice::{declare_dyn_slice, DynSliceMethods};

// Create our custom trait
pub trait MyTrait {
    fn to_u64(&self) -> u64;
    fn add(&self, rhs: u64) -> u64;
}

// Implement the trait for u8
impl MyTrait for u8 {
    fn to_u64(&self) -> u64 {
        u64::from(*self)
    }

    fn add(&self, rhs: u64) -> u64 {
        self.to_u64() + rhs
    }
}

// Implement the trait for u16
impl MyTrait for u16 {
    fn to_u64(&self) -> u64 {
        u64::from(*self)
    }

    fn add(&self, rhs: u64) -> u64 {
        self.to_u64() + rhs
    }
}

// Declare and import the `&dyn [MyTrait]` type
declare_dyn_slice!(MyTrait, my_trait_dyn_slice);
use my_trait_dyn_slice::*;

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    // Create the first dyn slice
    let dyn_slice = DynSlice::new(&array);

    // Get the first and last elements as u64
    let first = dyn_slice.first().map(MyTrait::to_u64);
    let last = dyn_slice.last().map(MyTrait::to_u64);

    println!("1: first: {first:?}, last: {last:?}");

    let array2: [u16; 3] = [5, 6, 7];

    // Create the second dyn slice
    let dyn_slice2 = DynSlice::new(&array2);

    // Get the first and last elements as u64
    let first = dyn_slice2.first().map(MyTrait::to_u64);
    let last = dyn_slice2.last().map(MyTrait::to_u64);

    println!("2: first: {first:?}, last: {last:?}\n");

    // Print the sum of each pair from the dyn slices
    let iter = dyn_slice.iter().zip(dyn_slice2.iter());
    for (i, (a, b)) in iter.enumerate() {
        println!("sum {}: {}", i + 1, a.add(b.to_u64()));
    }
}
