use dyn_slice::standard::debug;

fn main() {
    let a = [1, 2, 3, 4, 5, 6];
    let ds = debug::new(&a);

    let mut iter = ds.windows(3).unwrap();
    println!("{}", iter.len());

    while let Some(window) = iter.next() {
        println!("{window:?}, {}", iter.len());
    }
}
