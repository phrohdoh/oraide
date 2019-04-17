use crate::{
    computation::{
        ParserCtxExt,
        ParserCtxStorage,
    },
};

/// Entrypoint into MiniYaml parsing
///
/// Contains inputs and memoized computation results
///
/// # Example
/// ```rust
/// use oraide_parser_miniyaml::{Database,ParserCtx,ParserCtxExt,Tree};
/// let mut db = Database::default();
/// let file_id = db.add_file("example.yaml", "Hello:\n");
/// let tree: Tree = db.file_tree(file_id);
/// ```
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
