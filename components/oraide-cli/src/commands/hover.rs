use std::path::PathBuf;

use oraide_span::{
    FileId,
};

use oraide_query_system::{
    Database,
    LangServerCtx,
    Markdown,
    LsPos,
};

pub(crate) struct Hover {
    line_idx: u64,
    col_idx: u64,
    file_id: FileId,
    db: Database,
}

impl Hover {
    pub(crate) fn new(file_path: PathBuf, line_idx: u64, col_idx: u64) -> Result<Self, String> {
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
        match self.db.hover_with_file_id(self.file_id, LsPos::new(self.line_idx, self.col_idx)) {
            Some(Markdown(md)) => println!("{:?}", md),
            _ => println!("no results"),
        }
    }
}