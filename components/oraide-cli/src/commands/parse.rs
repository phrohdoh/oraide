use std::{
    path::{
        PathBuf,
    },
};

use oraide_span::FileId;
use oraide_parser_miniyaml::{
    Database,
    ParserCtx as _,
    TokenCollectionExts as _,
};

pub(crate) struct Parse {
    file_ids: Vec<FileId>,
    db: Database,
}

impl Parse {
    pub(crate) fn new(file_paths: Vec<PathBuf>) -> Result<Self, String> {
        let mut db = Database::default();

        let file_ids = file_paths.iter()
            .map(|path| crate::add_file(&mut db, path))
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

            let def_locs_and_slices = defs.iter()
                .filter_map(|shrd_node| shrd_node.key_tokens.span())
                .map(|span| {
                    let start = span.start();
                    let loc = self.db.location(*file_id, start);
                    let end_exclusive = span.end_exclusive().to_usize();
                    (loc, &text[start.to_usize()..end_exclusive])
                })
                .collect::<Vec<_>>();

            for (loc, slice) in def_locs_and_slices {
                println!(" - {} @ {}:{}", slice, file_name, loc);
            }
        }
    }
}