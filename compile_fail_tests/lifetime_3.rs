// Make sure that iterators from dyn slices do not live
// longer than the underlying slice / array

use dyn_slice::DynSlice;
use std::borrow::Borrow;

fn main() {
    let _ = {
        let array_1 = [1, 2, 3, 4, 5];
        let slice_1 = DynSlice::<dyn Borrow<i32>>::new(&array_1);
        slice_1.iter()
    };
}
