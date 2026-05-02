// TODO: where it would cause no extra operations, replace some slice manipulation for splitting / indexing methods

mod chunks;
mod chunks_mut;
#[expect(clippy::module_inception)]
mod iter;
mod iter_mut;
mod split;
mod split_mut;
pub use chunks::Chunks;
pub use chunks_mut::ChunksMut;
pub use iter::Iter;
pub use iter_mut::IterMut;
pub use split::Split;
pub use split_mut::SplitMut;
