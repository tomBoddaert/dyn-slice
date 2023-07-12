#![feature(ptr_metadata, pointer_byte_offsets)]

use dyn_slice::{declare_dyn_slice, DynSliceTrait};

pub trait MyTrait<T> {
    fn to_t(&self) -> T;
    fn add(&self, rhs: T) -> T;
}

macro_rules! impl_my_trait {
    ( $ty:ty, $to:ty ) => {
        impl MyTrait<$to> for $ty {
            fn to_t(&self) -> $to {
                u64::from(*self)
            }

            fn add(&self, rhs: $to) -> $to {
                self.to_t() + rhs
            }
        }
    };
}

impl_my_trait!(u8, u64);
impl_my_trait!(u16, u64);

declare_dyn_slice!(<T>, MyTrait:<T>, my_trait_dyn_slice);
use my_trait_dyn_slice::*;

declare_dyn_slice!(MyTrait:<u64>, my_trait_u64_dyn_slice);
#[allow(unused_imports)]
use my_trait_u64_dyn_slice::DynSlice as DynSliceMTU64;

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    let dyn_slice = DynSlice::<'_, u64>::new(&array);

    let first = dyn_slice.first().map(MyTrait::<u64>::to_t);
    let last = dyn_slice.last().map(MyTrait::<u64>::to_t);
    println!("1: first: {first:?}, last: {last:?}");

    let array2: [u16; 3] = [5, 6, 7];

    let dyn_slice2 = DynSlice::<'_, u64>::new(&array2);

    let first = dyn_slice2.first().map(MyTrait::<u64>::to_t);
    let last = dyn_slice2.last().map(MyTrait::<u64>::to_t);

    println!("2: first: {first:?}, last: {last:?}\n");

    let iter = dyn_slice.iter().zip(dyn_slice2.iter());
    for (i, (a, b)) in iter.enumerate() {
        println!("sum {}: {}", i + 1, a.add(b.to_t()));
    }
}
