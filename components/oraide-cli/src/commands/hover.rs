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

use std::path::PathBuf;

use oraide_span::{
    FileId,
};

use oraide_actor::{
    Position,
};

use oraide_query_system::{
    OraideDatabase,
};

use oraide_language_server::{
    LanguageServerCtx as _,
};

pub(crate) struct Hover {
    line_idx: usize,
    col_idx: usize,
    file_id: FileId,
    db: OraideDatabase,
}

impl Hover {
    pub(crate) fn new(
        root: PathBuf,
        rel_file_path: PathBuf,
        line_idx: usize,
        col_idx: usize,
    ) -> Result<Self, String> {
        let mut db = OraideDatabase::default();

        db.set_workspace_root(root.clone().into());
        let file_path = root.join(rel_file_path);
        let file_id = crate::add_file(&mut db, &file_path)?;

        Ok(Self {
            line_idx,
            col_idx,
            file_id,
            db,
        })
    }

    pub(crate) fn run(&self) {
        match self.db.documentation_for_position_in_file(
            self.file_id,
            Position::new(
                self.line_idx,
                self.col_idx,
            ),
        ) {
            Some(string) => println!("{:?}", string),
            _ => println!("no results"),
        }
    }
}