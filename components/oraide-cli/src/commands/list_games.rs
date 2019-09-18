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
// along with oraide.  If not, see <https://www.gnu.org/licenses/>

use {
    std::{
        path::PathBuf,
    },
    oraide_query_system::{
        OraideDatabase,
    },
    oraide_language_server::{
        LanguageServerCtx as _,
    },
    oraide_sdk::{
        SdkCtx as _,
    },
};

pub struct ListGames {
    db: OraideDatabase,
}

impl ListGames {
    pub fn new_with_root_dir(root_dir: PathBuf) -> Self {
        let mut db = OraideDatabase::default();
        db.set_workspace_root(root_dir.into());

        Self {
            db,
        }
    }

    pub fn run(&self) {
        let workspace_root = self.db.workspace_root().unwrap();
        let games = self.db.all_games().unwrap();

        for game in games {
            let game_root = workspace_root.join(game.rel_root_path);
            let manifest_path = game_root.join("mod.yaml");

            println!("{}:", game.id);
            println!("  manifest: {}", manifest_path.display());
            println!("     rules:");

            let resolved_rule_file_paths = match self.db.resolved_rule_file_paths_for_game(game.id.clone()) {
                Some(paths) => paths,
                _ => {
                    eprintln!("            Failed to get resolved rule file paths for `{}`", game.id);
                    continue;
                },
            };

            for path in resolved_rule_file_paths {
                println!("            {}", path.display());
            }

            println!();
        }
    }
}