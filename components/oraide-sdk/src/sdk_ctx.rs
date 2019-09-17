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

//! This module defines `SdkCtx` which is a `salsa` query group for SDK-based
//! game data such as:
//! - IDs of all games under a given workspace root directory

use {
    std::{
        fs,
        path::{
            PathBuf,
            Component,
        },
    },
    oraide_language_server::{
        LanguageServerCtx,
    },
    crate::{
        Game,
        GameId,
    },
};

// TODO: This *should not* rely on `oraide-language-server`.
#[salsa::query_group(SdkCtxStorage)]
pub trait SdkCtx: LanguageServerCtx {
    fn all_games(&self) -> Option<Vec<Game>>;

    fn resolved_rule_file_paths_for_game(&self, game_id: GameId) -> Option<Vec<PathBuf>>;
}

fn all_games(
    db: &impl SdkCtx,
) -> Option<Vec<Game>> {
    let root = db.workspace_root()?;
    let rel_mods_dir = root.join("mods/");
    let read_dir = fs::read_dir(&rel_mods_dir).ok()?;

    let games: Vec<_> = read_dir.filter_map(|entry| {
        let entry = entry.ok()?;
        let path = entry.path();

        // A manifest must exist and must be a file
        let manifest_path = path.join("mod.yaml");
        let md = fs::metadata(manifest_path).ok()?;
        if !md.is_file() {
            return None;
        }

        let id = match path.components().last()? {
            Component::Normal(os_str) => os_str.to_str()?.to_owned(),
            _ => return None,
        };

        let game = Game {
            id: id.into(),
        };

        game.into()
    }).collect();

    games.into()
}

fn resolved_rule_file_paths_for_game(
    _db: &impl SdkCtx,
    _game_id: GameId,
) -> Option<Vec<PathBuf>> {
    unimplemented!()
}