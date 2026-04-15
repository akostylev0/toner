//! Collection of hashmap-like **de**/**ser**ializable data structures
pub mod aug;
pub use aug::{Hashmap, HashmapE, HashmapNode};
mod hm_label;
pub(crate) mod iter;
pub mod pfx;
mod std;
