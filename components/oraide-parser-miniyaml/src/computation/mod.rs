//! # `computation`
//!
//! Exposes the parser's functionality wrapped in memoized computations
//!
//! # Example
//! ```rust
//! use oraide_parser_miniyaml::{Database,ParserCtx,ParserCtxExt,Tree};
//! let mut db = Database::default();
//! // note that the first argument is a name (which can be a path)
//! // but you must provide the content to turn into a tree,
//! // this package doesn't do that for you
//! let file_id = db.add_file("example.yaml", "Hello:\n");
//! let tree: Tree = db.file_tree(file_id);
//! ```
//!

use oraide_span::{
    FileId,
    FileSpan,
    ByteIndex,
    Location,
};

use crate::{
    Token,
    Node,
    Tree,
};

mod database;
pub use database::{
    Database,
};

mod query_definitions;

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

    /// Compute all line offsets in byte indicies
    #[salsa::invoke(query_definitions::line_offsets)]
    fn line_offsets(&self, file_id: FileId) -> Vec<usize>;

    /// Convert a [`ByteIndex`] into a [`Location`]
    ///
    /// [`ByteIndex`]: struct.ByteIndex.html
    /// [`Location`]: struct.Location.html
    #[salsa::invoke(query_definitions::location)]
    fn location(&self, file_id: FileId, index: ByteIndex) -> Location;

    /// Compute all of the [`Token`]s in a [`FileId`]
    ///
    /// [`Token`]: struct.Token.html
    /// [`FileId`]: struct.FileId.html
    #[salsa::invoke(query_definitions::file_tokens)]
    fn file_tokens(&self, file_id: FileId) -> Vec<Token>;

    /// Compute all of the [`Node`]s in a [`FileId`]
    ///
    /// [`Node`]: struct.Node.html
    /// [`FileId`]: struct.FileId.html
    #[salsa::invoke(query_definitions::file_nodes)]
    fn file_nodes(&self, file_id: FileId) -> Vec<Node>;

    /// Compute the [`Tree`] of a [`FileId`]
    ///
    /// [`Tree`]: struct.Tree.html
    /// [`FileId`]: struct.FileId.html
    #[salsa::invoke(query_definitions::file_tree)]
    fn file_tree(&self, file_id: FileId) -> Tree;

    /// Compute the definitions (top-level items) of a [`FileId`]
    ///
    /// This is, essentially, the file's symbol table
    ///
    /// [`FileId`]: struct.FileId.html
    #[salsa::invoke(query_definitions::file_definitions)]
    fn file_definitions(&self, file_id: FileId) -> Vec<Node>;

    /// Compute the definitions (top-level items) of all files
    ///
    /// This is, essentially, a symbole table for all files
    ///
    /// # Returns
    /// For each file:
    /// - The file's ID
    /// - The file's definitions
    #[salsa::invoke(query_definitions::all_definitions)]
    fn all_definitions(&self) -> Vec<(FileId, Vec<Node>)>;

    /// Compute the span of a definition with the given name in a particular file
    #[salsa::invoke(query_definitions::file_definition_span)]
    fn file_definition_span(&self, file_id: FileId, def_name: String) -> Option<FileSpan>;
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

impl ParserCtxExt for database::Database {}