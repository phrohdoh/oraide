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
        fs::{
            self,
            DirEntry,
        },
        path::{
            PathBuf,
            Component,
        },
    },
    oraide_span::{
        FileId,
    },
    oraide_parser_miniyaml::{
        Tree,
        TextFilesCtxExt,
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

    fn game_root_dir(
        &self,
        game_id: GameId,
    ) -> Option<PathBuf>;

    fn game_file_id(
        &self,
        game_id: GameId,
        rel_file_path: PathBuf,
    ) -> Option<FileId>;

    fn game_manifest_file_path(
        &self,
        game_id: GameId,
    ) -> Option<PathBuf>;

    fn game_manifest_file_id(
        &self,
        game_id: GameId,
    ) -> Option<FileId>;

    fn game_manifest_tree(
        &self,
        game_id: GameId,
    ) -> Option<Tree>;

    fn game_manifest_rules_entries(
        &self,
        game_id: GameId,
    ) -> Option<Vec<String>>;

    fn resolve_path_for_game(
        &self,
        game_id: GameId,
        unresolved_path: String,
    ) -> Option<PathBuf>;

    fn resolved_rule_file_paths_for_game(
        &self,
        game_id: GameId,
    ) -> Option<Vec<PathBuf>>;
}

fn all_games(
    db: &impl SdkCtx,
) -> Option<Vec<Game>> {
    fn dir_entry_to_game(entry: std::io::Result<DirEntry>) -> Option<Game> {
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
            rel_root_path: path,
        };

        game.into()
    }

    let root = db.workspace_root()?;

    // Read `Game`s from the top-level "mods" directory (engine and sdk).
    let rel_mods_dir = root.join("mods/");
    let read_mods_dir = fs::read_dir(&rel_mods_dir).ok()?;

    let mut top_level_games: Vec<_> = read_mods_dir
        .filter_map(dir_entry_to_game)
        .collect();

    // Read `Game`s from the engine submodule, if it exists.
    let engine_dir_games_opt: Option<Vec<Game>> = {
        let rel_engine_mods_dir = root.join("engine/mods/");
        let read_engine_mods_dir_opt = fs::read_dir(&rel_engine_mods_dir).ok();

        read_engine_mods_dir_opt.map(|read_dir|
            read_dir.filter_map(dir_entry_to_game)
                .collect()
        )
    };

    match engine_dir_games_opt {
        Some(mut engine_games) => {
            engine_games.append(&mut top_level_games);
            engine_games
        },
        _ => top_level_games,
    }.into()
}

/// Get the `rel_root_path` of the `Game` where `id == game_id`
fn game_root_dir(
    db: &impl SdkCtx,
    game_id: GameId,
) -> Option<PathBuf> {
    let game = db.all_games()?
        .into_iter()
        .find(|game| game.id == game_id)?;

    game.rel_root_path.into()
}

fn game_file_id(
    db: &impl SdkCtx,
    game_id: GameId,
    rel_file_path: PathBuf,
) -> Option<FileId> {
    let game_root_dir = db.game_root_dir(game_id)?;
    let file_path = game_root_dir.join(rel_file_path)
        .into_os_string()
        .into_string()
        .ok()?;

    let file_id = db.file_id_of_file_path(file_path)?;
    file_id.into()
}

fn game_manifest_file_path(
    db: &impl SdkCtx,
    game_id: GameId,
) -> Option<PathBuf> {
    let game_root_dir = db.game_root_dir(game_id)?;
    game_root_dir.join("mod.yaml").into()
}

/// Shorthand for `db.game_file_id(game_id, "mod.yaml")`
fn game_manifest_file_id(
    db: &impl SdkCtx,
    game_id: GameId,
) -> Option<FileId> {
    let file_id = match db.game_file_id(game_id, "mod.yaml".into()) {
        Some(id) => id,
        _ => {
            let file_path = db.game_manifest_file_path(game_id)?;
            let file_text = fs::read_to_string(&file_path).ok()?;

            // NEXT: https://salsa.zulipchat.com/#narrow/stream/145099-general/topic/general.20questions/near/176007317

            file_id
        },
    };

    file_id.into()
}

/// Get the [`Tree`] of the manifest of the `Game` where `id == game_id`
///
/// [`Tree`]: ../oraide_parser_miniyaml/struct.Tree.html
fn game_manifest_tree(
    db: &impl SdkCtx,
    game_id: GameId,
) -> Option<Tree> {
    let manifest_file_id = db.game_manifest_file_id(game_id)?;
    let manifest_tree = db.file_tree(manifest_file_id)?;
    manifest_tree.into()
}

/// Get the key text of the children of the `Rules` node of a `Game`
fn game_manifest_rules_entries(
    db: &impl SdkCtx,
    game_id: GameId,
) -> Option<Vec<String>> {
    let file_id = db.game_manifest_file_id(game_id.clone())?;
    let file_text = db.file_text(file_id)?;
    let tree = db.game_manifest_tree(game_id)?;

    let rules_arena_node_id = tree.find_node(|node|
        node.is_top_level() && node.key_text(&file_text) == "Rules".into()
    )?;

    let rules_key_texts: Vec<_> = rules_arena_node_id.children(&tree.arena)
        .filter_map(|child_node_id| {
            let arena_node: _ = tree.arena.get(child_node_id)?;
            let child_node_key_text_opt = arena_node.get()
                .key_text(&file_text)
                .map(|shrd_str| shrd_str.to_owned());

            child_node_key_text_opt
        }).collect();

    rules_key_texts.into()
}

/// Resolve a relative file path for a particular game.
/// 
/// # Examples
/// 
/// Assume the following file structure.
/// 
/// ```
/// <workspace root> # let's assume this is `/home/you/my-cool-game`
/// └── mods/
///    ├── bar/
///    │  ├── mod.yaml
///    │  └── rules/
///    │     └── civilian-structures.yaml
///    └── foo/
///       └── mod.yaml
/// ```
/// 
/// Assume the following `<workspace root>/mods/foo/mod.yaml` contents.
/// 
/// ```
/// Packages:
///     $bar: bar
/// 
/// Rules:
///     bar|rules/civilian-structures.yaml
/// ```
/// 
/// To determine the file path of `civilian-structures.yaml` we need to resolve
/// the `bar|` prefix to a real path.
/// 
/// The `$bar: bar` node in the `Packages` section is saying "the prefix `bar`
/// refers to the root directory of the `bar` game ('mod' in OpenRA terms)."
/// 
/// You can do that like so (ignoring any missing values / errors for
/// simple demonstration purposes, your code should absolutely handle errors,
/// meaning don't blindly call `unwrap`).
/// 
/// ```rust,no_run
/// use oraide_query_system::OraideDatabase;
/// use oraide_sdk::SdkCtx as _;
///
/// let db = OraideDatabase::default();
/// let path = db.resolve_path_for_game(
///     String::from("foo").into(),
///     "bar|rules/civilian-structures.yaml",
/// ).unwrap();
/// 
/// assert_eq(path, "/home/you/my-cool-game/mods/bar/rules/civilian-structures.yaml");
/// ```
/// You could use this resolved path to, for example, determine whether an actor
/// is defined in the given file.
///
/// # Limitations
///
/// Currently this implementation does not consider package 'aliases'.
///
/// The implication of this is that `cnc|rules/ai.yaml`, for example, will
/// assume that `cnc` is the actual ID of a game where in reality it _could_ be
/// defined as `$cnc: foobar` in the `Packages` section of the manifest.
fn resolve_path_for_game(
    db: &impl SdkCtx,
    _game_id: GameId,
    unresolved_path: String,
) -> Option<PathBuf> {
    let (refd_game_id, rel_path) = {
        let mut split = unresolved_path.splitn(2, '|');
        let refd_game_id = GameId(split.next()?.to_owned());
        (refd_game_id, split.next()?)
    };

    let refd_game_root = db.game_root_dir(refd_game_id)?;
    refd_game_root.join(rel_path).into()
}

fn resolved_rule_file_paths_for_game(
    db: &impl SdkCtx,
    game_id: GameId,
) -> Option<Vec<PathBuf>> {
    let entries = db.game_manifest_rules_entries(game_id.clone())?;

    entries.into_iter()
        .map(|entry| db.resolve_path_for_game(game_id.clone(), entry))
        .collect()
}