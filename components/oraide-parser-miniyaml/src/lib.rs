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