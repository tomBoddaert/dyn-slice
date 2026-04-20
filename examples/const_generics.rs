use dyn_slice::DynSlice;

// Declare a trait with a constant parameter
pub trait AddConst<const N: u8> {
    fn add(&self) -> u8;
}

// Implement the trait for u8
impl<const N: u8> AddConst<N> for u8 {
    fn add(&self) -> u8 {
        self + N
    }
}

fn main() {
    // Create an array of u8
    let array = [5, 58, 97];
    // Create a dyn slice from the array
    let slice = DynSlice::<dyn AddConst<12>>::new(&array);

    // Add the numbers
    let sums = slice.iter().map(AddConst::add);
    // Print the results
    println!("{:?}", sums.collect::<Vec<u8>>());
}

// Test the example (this can be ignored)
#[test]
fn test() {
    main();
}
