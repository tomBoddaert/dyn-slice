// Make sure that the returned slice from downcasting a dyn slice
// does not live longer than the original slice

use dyn_slice::DynSlice;
use std::any::Any;

fn main() {
    let _ = {
        let array_1 = [1, 2, 3, 4, 5];
        let slice_1 = DynSlice::<dyn Any>::new(&array_1);
        slice_1.downcast_slice::<i32>().unwrap()
    };
}
