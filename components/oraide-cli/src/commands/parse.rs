use std::{
    io::{
        Read as _,
    },
    fs::{
        File,
    },
    path::{
        Path,
        PathBuf,
    },
};

use oraide_span::FileId;
use oraide_parser_miniyaml::{
    Database,
    ParserCtx as _,
    ParserCtxExt as _,
    TokenCollectionExts as _,
};

pub(crate) struct Parse {
    file_ids: Vec<FileId>,
    db: Database,
}

impl Parse {
    fn add_file(db: &mut Database, file_path: &Path) -> Result<FileId, String> {
        let text = {
            let mut file = File::open(file_path)
                .map_err(|e| format!("Error opening `{}`: {}", file_path.display(), e))?;

            let mut text = String::new();
            file.read_to_string(&mut text)
                .map_err(|e| format!("Error reading `{}`: {}", file_path.display(), e))?;

            text
        };

        let file_id = db.add_file(file_path.to_string_lossy(), text);

        Ok(file_id)
    }

    pub(crate) fn new(file_paths: Vec<PathBuf>) -> Result<Self, String> {
        let mut db = Database::default();

        let file_ids = file_paths.iter()
            .map(|path| Self::add_file(&mut db, path))
            .collect::<Result<_, String>>()?;

        Ok(Self {
            file_ids,
            db,
        })
    }

    pub(crate) fn run(&self) {
        for file_id in self.file_ids.iter() {
            let text = self.db.file_text(*file_id);
            let file_name = self.db.file_name(*file_id);
            let defs = self.db.file_definitions(*file_id);

            println!("Found {} definition(s) in {} ({:?})", defs.len(), file_name, *file_id);

            let def_slices = defs.iter()
                .filter_map(|shrd_node| shrd_node.key_tokens.span())
                .map(|span| {
                    let start = span.start().to_usize();
                    let end_exclusive = span.end_exclusive().to_usize();
                    &text[start..end_exclusive]
                })
                .collect::<Vec<_>>();

            for slice in def_slices {
                println!(" - {}", slice);
            }
        }
    }
}