#[expect(clippy::module_inception)]
mod iter;
mod iter_mut;
mod split;
mod split_mut;
pub use iter::Iter;
pub use iter_mut::IterMut;
pub use split::Split;
pub use split_mut::SplitMut;
