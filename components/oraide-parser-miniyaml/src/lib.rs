// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Convert textual MiniYaml documents into MiniYaml trees
//!
//! # Examples
//!
//! ```rust
//! //
//! ```
//!
//! See [`Database`] docs for an example with [`oraide-query-system`]
//!
//! [`oraide-query-system`]: ../oraide_query_system/index.html
//! [`Database`]: ../oraide_query_system/struct.Database.html

mod parser;
pub use parser::{
    Token,
    TokenKind,
    Tokenizer,
    TokenCollectionExts,
    Node,
    Nodeizer,
    IndentLevelDelta,
    Arena,
    ArenaNodeId,
    Tree,
    Treeizer,
};

mod computation;
pub use computation::{
    ParserCtx,
    ParserCtxExt,
    ParserCtxStorage,
};