// Make sure that dyn slices do not live longer than the underlying slice / array

use dyn_slice::DynSlice;
use std::borrow::Borrow;

fn main() {
    let array_1 = [1, 2, 3, 4, 5];
    let slice_1 = DynSlice::<dyn Borrow<i32>>::new(&array_1);

    let slice_2 = {
        let array_2 = [6, 7, 8, 9, 10];
        DynSlice::<dyn Borrow<i32>>::new(&array_2)
    };

    let _ = (&slice_1, &slice_2);
}
