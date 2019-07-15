use std::path::PathBuf;

use oraide_span::{
    FileId,
};

use oraide_query_system::{
    Database,
    LangServerCtx,
    Markdown,
    Position,
};

pub(crate) struct Hover {
    line_idx: usize,
    col_idx: usize,
    file_id: FileId,
    db: Database,
}

impl Hover {
    pub(crate) fn new(file_path: PathBuf, line_idx: usize, col_idx: usize) -> Result<Self, String> {
        let mut db = Database::default();

        let file_id = crate::add_file(&mut db, &file_path)?;

        Ok(Self {
            line_idx,
            col_idx,
            file_id,
            db,
        })
    }

    pub(crate) fn run(&self) {
        match self.db.hover_with_file_id(self.file_id, Position::new(self.line_idx, self.col_idx)) {
            Some(Markdown(md)) => println!("{:?}", md),
            _ => println!("no results"),
        }
    }
}