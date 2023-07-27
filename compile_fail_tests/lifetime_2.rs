// Make sure that references from dyn slices do not live
// longer than the underlying slice / array

use dyn_slice::standard::borrow;

fn main() {
    let _ = {
        let array_1 = [1, 2, 3, 4, 5];
        let slice_1 = borrow::new(&array_1);
        slice_1.get(1).unwrap()
    };
}
