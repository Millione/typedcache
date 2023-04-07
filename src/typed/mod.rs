//! Typed key-value expression, borrowed from [typedmap](https://github.com/kodieg/typedmap).

pub mod dynhash;
pub mod typedkey;
pub mod typedvalue;

use std::hash::Hash;

pub trait TypedMap: Eq + Hash {
    type Value: 'static;
}
