// Make sure that a mutable dyn slice cannot create a mutable iterator
// then access the slice while the iterator is in scope

use dyn_slice::standard::borrow;

fn main() {
    let mut array_1 = [1, 2, 3, 4, 5];
    let mut slice_1 = borrow::new_mut(&mut array_1);

    let mut a = slice_1.iter_mut();

    let _ = slice_1.get(1).unwrap().borrow();

    a.next().unwrap().borrow();
}
