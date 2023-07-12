#![feature(ptr_metadata, pointer_byte_offsets)]

use dyn_slice::{declare_dyn_slice, DynSliceTrait};

pub trait MyTrait {
    fn to_u64(&self) -> u64;
    fn add(&self, rhs: u64) -> u64;
}

macro_rules! impl_my_trait {
    ( $ty:ty ) => {
        impl MyTrait for $ty {
            fn to_u64(&self) -> u64 {
                u64::from(*self)
            }

            fn add(&self, rhs: u64) -> u64 {
                self.to_u64() + rhs
            }
        }
    };
}

impl_my_trait!(u8);
impl_my_trait!(u16);

declare_dyn_slice!(MyTrait, my_trait_dyn_slice);
use my_trait_dyn_slice::*;

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    let dyn_slice = DynSlice::new(&array);

    let first = dyn_slice.first().map(MyTrait::to_u64);
    let last = dyn_slice.last().map(MyTrait::to_u64);

    println!("1: first: {first:?}, last: {last:?}");

    let array2: [u16; 3] = [5, 6, 7];

    let dyn_slice2 = DynSlice::new(&array2);

    let first = dyn_slice2.first().map(MyTrait::to_u64);
    let last = dyn_slice2.last().map(MyTrait::to_u64);

    println!("2: first: {first:?}, last: {last:?}\n");

    let iter = dyn_slice.iter().zip(dyn_slice2.iter());
    for (i, (a, b)) in iter.enumerate() {
        println!("sum {}: {}", i + 1, a.add(b.to_u64()));
    }
}
