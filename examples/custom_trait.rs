use dyn_slice::DynSlice;

// Create our custom trait
pub trait MyTrait {
    fn to_u64(&self) -> u64;
    fn add(&self, rhs: u64) -> u64;
}

// Implement the trait for u8
impl MyTrait for u8 {
    fn to_u64(&self) -> u64 {
        u64::from(*self)
    }

    fn add(&self, rhs: u64) -> u64 {
        self.to_u64() + rhs
    }
}

// Implement the trait for u16
impl MyTrait for u16 {
    fn to_u64(&self) -> u64 {
        u64::from(*self)
    }

    fn add(&self, rhs: u64) -> u64 {
        self.to_u64() + rhs
    }
}

fn main() {
    let array: [u8; 4] = [1, 2, 3, 4];

    // Create the first dyn slice
    let slice = DynSlice::<dyn MyTrait>::new(&array);

    // Get the first and last elements as u64
    let first = slice.first().map(MyTrait::to_u64);
    let last = slice.last().map(MyTrait::to_u64);

    println!("1: first: {first:?}, last: {last:?}");

    let array2: [u16; 3] = [5, 6, 7];

    // Create the second dyn slice
    let slice2 = DynSlice::<dyn MyTrait>::new(&array2);

    // Get the first and last elements as u64
    let first = slice2.first().map(MyTrait::to_u64);
    let last = slice2.last().map(MyTrait::to_u64);

    println!("2: first: {first:?}, last: {last:?}\n");

    // Print the sum of each pair from the dyn slices
    let iter = slice.iter().zip(slice2.iter());
    for (i, (a, b)) in iter.enumerate() {
        println!("sum {}: {}", i + 1, a.add(b.to_u64()));
    }
}

// Test the example (this can be ignored)
#[test]
fn test() {
    main()
}
