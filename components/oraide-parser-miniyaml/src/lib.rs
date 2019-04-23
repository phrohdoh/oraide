// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! # `oraide-parser-miniyaml`
//!
//! Convert textual MiniYaml documents into MiniYaml trees
//!
//! # Example
//! ```rust
//! # use oraide_parser_miniyaml::{Database,ParserCtx,ParserCtxExt,Tree};
//! let mut db = Database::default();
//! let file_id = db.add_file("example.yaml", "Hello:\n");
//! let tree: Tree = db.file_tree(file_id);
//! ```

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
    Database,
    ParserCtx,
    ParserCtxExt,
};