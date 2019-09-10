// This file is part of oraide.  See <https://github.com/Phrohdoh/oraide>.
// 
// oraide is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3
// as published by the Free Software Foundation.
// 
// oraide is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with oraide.  If not, see <https://www.gnu.org/licenses/>.

use std::{
    path::{
        PathBuf,
    },
};

use oraide_span::FileId;

use oraide_parser_miniyaml::{
    TokenCollectionExts as _,
    FilesCtx as _,
    TextFilesCtx as _,
    ParserCtx as _,
};

use oraide_query_system::OraideDatabase;

pub(crate) struct Parse {
    file_ids: Vec<FileId>,
    db: OraideDatabase,
}

impl Parse {
    pub(crate) fn new(file_paths: Vec<PathBuf>) -> Result<Self, String> {
        let mut db = OraideDatabase::default();

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
            let text = match self.db.file_text(*file_id) {
                Some(text) => text,
                _ => continue,
            };

            let file_path = match self.db.file_path(*file_id) {
                Some(path) => path,
                _ => continue,
            };

            let top_level_nodes = match self.db.all_top_level_nodes_in_file(*file_id) {
                Some(nodes) => nodes,
                _ => continue,
            };

            println!("Found {} definition(s) in {} ({:?})", top_level_nodes.len(), file_path, *file_id);

            let def_locs_and_slices = top_level_nodes.iter()
                .filter_map(|shrd_node| shrd_node.key_tokens.span())
                .map(|span| {
                    let start = span.start();
                    let loc = self.db.convert_byte_index_to_location(*file_id, start).unwrap();
                    let end_exclusive = span.end_exclusive().to_usize();
                    (loc, &text[start.to_usize()..end_exclusive])
                })
                .collect::<Vec<_>>();

            for (loc, slice) in def_locs_and_slices {
                println!(" - {} @ {}:{}", slice, file_path, loc);
            }
        }
    }
}