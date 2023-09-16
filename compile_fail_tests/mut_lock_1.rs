// Make sure that a mutable dyn slice cannot hand out a mutable
// reference to the data while there is also an immutable slice

use dyn_slice::standard::borrow_mut;

fn main() {
    let mut array_1 = [1, 2, 3, 4, 5];
    let mut slice_1 = borrow_mut::new_mut(&mut array_1);

    let slice_2 = slice_1.slice(0..2).unwrap();

    let a = slice_1.get_mut(3).unwrap().borrow_mut();
    *a = 6;

    let _ = (&slice_1, &slice_2);
}
