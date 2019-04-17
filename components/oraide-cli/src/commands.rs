use std::{
    io::{
        Read as _,
    },
    fs::{
        File,
    },
    path::{
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

pub struct Parse {
    file_id: FileId,
    db: Database,
}

impl Parse {
    pub(crate) fn new(file_path: PathBuf) -> Result<Self, String> {
        let mut db = Database::default();

        let text = {
            let mut file = File::open(&file_path)
                .map_err(|e| format!("Error opening `{}`: {}", file_path.display(), e))?;

            let mut text = String::new();
            file.read_to_string(&mut text)
                .map_err(|e| format!("Error reading `{}`: {}", file_path.display(), e))?;

            text
        };

        let file_id = db.add_file(file_path.to_string_lossy(), text);

        Ok(Self {
            file_id,
            db,
        })
    }

    pub(crate) fn run(&self) {
        let text = self.db.file_text(self.file_id);
        let file_name = self.db.file_name(self.file_id);
        let defs = self.db.file_definitions(self.file_id);

        println!("Found {} definition(s) in `{}`:", defs.len(), file_name);

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