// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    path::{
        PathBuf,
    },
};

use oraide_span::FileId;

use oraide_query_system::OraideDatabase;

use oraide_parser_miniyaml::{
    ParserCtx as _,
};

pub(crate) struct FindDefinition {
    name_to_find: String,
    file_ids: Vec<FileId>,
    db: OraideDatabase,
}

impl FindDefinition {
    pub(crate) fn new(name_to_find: String, project_root_dir: PathBuf) -> Result<Self, String> {
        let mut db = OraideDatabase::default();

        let dir_walker = walkdir::WalkDir::new(&project_root_dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.metadata().map(|md| md.is_file()).unwrap_or(false))
            .filter(|entry| entry.path().extension() == Some(std::ffi::OsString::from("yaml".to_string()).as_ref()))
            ;

        let file_ids = dir_walker
            .map(|file| crate::add_file(&mut db, file.path()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            name_to_find,
            file_ids,
            db,
        })
    }

    pub(crate) fn run(&self) {
        for file_id in self.file_ids.iter() {
            if let Some(span) = self.db.file_definition_span(*file_id, self.name_to_find.clone()) {
                let file_name = self.db.file_name(*file_id);
                let start = span.start();
                let loc = self.db.location(*file_id, start);
                println!("{}:{}", file_name, loc);
            }
        }
    }
}