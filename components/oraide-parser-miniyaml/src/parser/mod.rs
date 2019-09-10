// This file is part of oraide.  See <https://github.com/Phrohdoh/oraide>.
// 
// oraide is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3
// as published by the Free Software Foundation.
// 
// oraide is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with oraide.  If not, see <https://www.gnu.org/licenses/>.

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

pub use nodeizer::{
    Node,
    Nodeizer,
};

pub use treeizer::{
    Tree,
    Treeizer,
    IndentLevelDelta,
    Arena,
    ArenaNodeId,
};