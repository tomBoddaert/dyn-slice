// Make sure that a mutable reference cannot be created while
// there a reference to an element of an iterator created with `Iterator::last`

use dyn_slice::standard::borrow;

fn main() {
    let mut array_1 = [1, 2, 3, 4, 5];
    let mut slice_1 = borrow::new_mut(&mut array_1);
    let iter_1 = slice_1.iter();
    let last = iter_1.last().unwrap();

    let _ = slice_1.get_mut(0).unwrap().borrow();

    let _ = last.borrow();
}
