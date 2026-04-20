// Make sure that a mutable dyn slice cannot hand out a mutable
// dyn slice while there is also an immutable slice

use dyn_slice::DynSliceMut;
use std::borrow::BorrowMut;

fn main() {
    let mut array_1 = [1, 2, 3, 4, 5];
    let mut slice_1 = DynSliceMut::<dyn BorrowMut<i32>>::new(&mut array_1);

    let slice_2 = slice_1.coerce_shared().get(0..2).unwrap();

    let _ = slice_1.get_mut(2..4).unwrap();

    let _ = (&slice_1, &slice_2);
}
