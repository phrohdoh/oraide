//! # `oraide-parser-miniyaml`
//!
//! Convert textual MiniYaml documents into MiniYaml trees
//!

use indextree::{
    NodeId as ArenaNodeId,
};

pub type Arena = indextree::Arena<Node>;

mod parser;

pub use parser::{
    Token,
    TokenKind,
    Tokenizer,
    TokenCollectionExts,
    Node,
    Nodeizer,
    IndentLevelDelta,
    Treeizer,
};

/// A [`Tree`] groups an [`indextree::Arena`] with all of its [`indextree::NodeId`]s
///
/// [`Tree`]: struct.Tree.html
/// [`indextree::Arena`]: ../indextree/struct.Arena.html
#[derive(Debug, Clone)]
pub struct Tree {
    /// All IDs for nodes that exist in `arena` with the first item always
    /// being the sentinel for parent-less nodes
    pub node_ids: Vec<ArenaNodeId>,

    /// The `indextree::Arena` that contains `Node`s
    pub arena: Arena,
}

impl Tree {
    pub fn from(node_ids: Vec<ArenaNodeId>, arena: Arena) -> Self {
        Self {
            node_ids,
            arena,
        }
    }
}