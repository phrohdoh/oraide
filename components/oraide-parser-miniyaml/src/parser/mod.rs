//! # `parser`
//!
//! This module consists of 3 sub-modules:
//!
//! - `tokenizer`
//!     - input: text
//!     - output: collection of `Token`s
//! - `nodeizer`
//!     - input: collection of `Token`s
//!     - output: collection of `Node`s
//! - `treeizer`
//!     - input: collection of `Node`s
//!     - output: a `Tree`
//!
//! It also contains types used by the previously-mentioned sub-modules
//! and other components of this project such as `Token`, `Node`, `Tree`, etc.
//!

mod tokenizer;
mod nodeizer;
mod treeizer;

pub use tokenizer::{
    Token,
    TokenKind,
    Tokenizer,
    TokenCollectionExts,
};

pub use nodeizer::Nodeizer;

pub use treeizer::{
    Treeizer,
    IndentLevelDelta,
};