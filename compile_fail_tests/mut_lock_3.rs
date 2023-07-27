// Make sure that a mutable dyn slice cannot hand out a mutable
// dyn slice while there is also an immutable borrow

use dyn_slice::standard::borrow;

fn main() {
    let mut array_1 = [1, 2, 3, 4, 5];
    let mut slice_1 = borrow::new_mut(&mut array_1);

    let element = slice_1.get(1).unwrap();

    let _ = slice_1.slice_mut(2..4).unwrap();

    let _ = element.borrow();
}
