// Make sure that a mutable dyn slice cannot hand out a mutable
// dyn slice while there is also an immutable slice

use dyn_slice::standard::borrow_mut;

fn main() {
    let mut array_1 = [1, 2, 3, 4, 5];
    let mut slice_1 = borrow_mut::new_mut(&mut array_1);

    let slice_2 = slice_1.slice(0..2).unwrap();

    let _ = slice_1.slice_mut(2..4).unwrap();

    let _ = slice_1.iter().chain(slice_2.iter());
}
