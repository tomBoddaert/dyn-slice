// Make sure that a mutable reference cannot be created while
// there is a copy of an immutable reference

use dyn_slice::standard::borrow;

fn main() {
    let mut array_1 = [1, 2, 3, 4, 5];
    let mut slice_1 = borrow::new_mut(&mut array_1);

    let slice_3 = {
        let slice_2 = slice_1.slice(..).unwrap();
        *&slice_2
    };

    let _ = slice_1.get_mut(0).unwrap();

    let _ = slice_3.get(1).unwrap();
}
