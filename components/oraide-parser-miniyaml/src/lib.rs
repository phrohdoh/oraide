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

use oraide_span::{
    FileId,
};

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

#[salsa::database(ParserCtxStorage)]
pub struct Database {
    rt: salsa::Runtime<Self>,
}

impl salsa::Database for Database {
    fn salsa_runtime(&self) -> &salsa::Runtime<Self> {
        &self.rt
    }
}

impl Default for Database {
    fn default() -> Self {
        let mut db = Self {
            rt: salsa::Runtime::default(),
        };

        db.init();
        db
    }
}

/// Provides MiniYaml-parsing inputs & queries
#[salsa::query_group(ParserCtxStorage)]
pub trait ParserCtx: salsa::Database {
    /// Text of the file that was assigned a given [`FileId`]
    ///
    /// [`FileId`]: struct.FileId.html
    #[salsa::input]
    fn file_text(&self, file_id: FileId) -> String;

    /// Name of the file that was assigned a given [`FileId`]
    ///
    /// [`FileId`]: struct.FileId.html
    #[salsa::input]
    fn file_name(&self, file_id: FileId) -> String;

    /// All of the tracked [`FileId`]s
    ///
    /// [`FileId`]: struct.FileId.html
    #[salsa::input]
    fn all_file_ids(&self) -> Vec<FileId>;

    /// Compute all of the [`Token`]s in a [`FileId`]
    ///
    /// [`Token`]: struct.Token.html
    /// [`FileId`]: struct.FileId.html
    fn file_tokens(&self, file_id: FileId) -> Vec<Token>;

    /// Compute all of the [`Node`]s in a [`FileId`]
    ///
    /// [`Node`]: struct.Node.html
    /// [`FileId`]: struct.FileId.html
    fn file_nodes(&self, file_id: FileId) -> Vec<Node>;

    /// Compute the [`Tree`] of a [`FileId`]
    ///
    /// [`Tree`]: struct.Tree.html
    /// [`FileId`]: struct.FileId.html
    fn file_tree(&self, file_id: FileId) -> Tree;
}

pub trait ParserCtxExt: ParserCtx {
    fn init(&mut self) {
        self.set_all_file_ids(Default::default());
    }

    /// Add a file to the database
    ///
    /// # Example
    /// ```rust
    /// # use oraide_parser_miniyaml::{Database,ParserCtx,ParserCtxExt};
    /// let mut db = Database::default();
    /// let text = "Hello:\n";
    /// let file_id = db.add_file("example.yaml", text);
    /// assert_eq!(text, db.file_text(file_id));
    /// ```
    ///
    /// # Returns
    /// A newly-created [`FileId`] that uniquely represents this file in this context
    ///
    /// [`FileId`]: struct.FileId.html
    fn add_file(&mut self, file_name: impl Into<String>, file_text: impl Into<String>) -> FileId {
        let file_name = file_name.into();
        let file_text = file_text.into();

        let mut all_file_ids = self.all_file_ids();
        let file_id = FileId(all_file_ids.len());
        all_file_ids.extend(Some(file_id));

        self.set_file_name(file_id, file_name);
        self.set_all_file_ids(all_file_ids);
        self.set_file_text(file_id, file_text);

        file_id
    }
}

impl ParserCtxExt for Database {}

fn file_tokens(db: &impl ParserCtx, file_id: FileId) -> Vec<Token> {
    let text = db.file_text(file_id);
    let mut tokenizer = Tokenizer::new(file_id, &text);
    tokenizer.run()
}

fn file_nodes(db: &impl ParserCtx, file_id: FileId) -> Vec<Node> {
    let tokens = db.file_tokens(file_id);
    let mut nodeizer = Nodeizer::new(tokens.into_iter());
    nodeizer.run()
}

fn file_tree(db: &impl ParserCtx, file_id: FileId) -> Tree {
    let text = db.file_text(file_id);
    let nodes = db.file_nodes(file_id);
    let mut treeizer = Treeizer::new(nodes.into_iter(), &text);
    treeizer.run()
}