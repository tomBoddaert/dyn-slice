// Make sure that a mutable dyn slice cannot hand out a mutable
// iterator while there is also an immutable borrow

use dyn_slice::standard::borrow;

fn main() {
    let mut array_1 = [1, 2, 3, 4, 5];
    let mut slice_1 = borrow::new_mut(&mut array_1);

    let element = slice_1.get(1).unwrap();

    let _ = slice_1.iter_mut();

    let _ = element.borrow();
}
