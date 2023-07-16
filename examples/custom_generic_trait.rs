// Enable the required features (nightly must be used)
#![feature(ptr_metadata, pointer_byte_offsets)]

// Remember to import the `DynSliceMethods` trait to use the methods!
use dyn_slice::{declare_dyn_slice, DynSliceMethods};

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

// Declare and import the `&dyn [MyTrait<T>]` type, generic on `T`
declare_dyn_slice!(<T>, MyTrait:<T>, my_trait_dyn_slice);
use my_trait_dyn_slice::*;

// Declare and import the `&dyn [MyTrait<u64>]` type
declare_dyn_slice!(MyTrait:<u64>, my_trait_u64_dyn_slice);
#[allow(unused_imports)]
use my_trait_u64_dyn_slice::DynSlice as DynSliceMTU64;

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    // Create the first dyn slice
    let dyn_slice = DynSlice::<'_, u64>::new(&array);

    // Get the first and last elements as u64
    let first = dyn_slice.first().map(MyTrait::<u64>::to_t);
    let last = dyn_slice.last().map(MyTrait::<u64>::to_t);
    println!("1: first: {first:?}, last: {last:?}");

    let array2: [u16; 3] = [5, 6, 7];

    // Create the second dyn slice
    let dyn_slice2 = DynSlice::<'_, u64>::new(&array2);

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
