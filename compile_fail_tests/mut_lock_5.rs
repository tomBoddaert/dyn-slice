// Make sure that a mutable dyn slice cannot hand out a mutable
// reference from an iterator while there is also an immutable borrow

use dyn_slice::DynSliceMut;
use std::borrow::BorrowMut;

fn main() {
    let mut array_1 = [1, 2, 3, 4, 5];
    let mut slice_1 = DynSliceMut::<dyn BorrowMut<i32>>::new(&mut array_1);

    let a = slice_1.reborrow().iter_mut().next().unwrap();

    let element = slice_1.coerce_shared().get(1).unwrap();

    a.borrow();

    let _ = element.borrow();
}
