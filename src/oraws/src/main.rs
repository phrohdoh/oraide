// invoke like so:
// cargo run -- ~/src/games/openra/engine/

use std::{
    env,
    fs,
    path::Path,
    io::Read as _,
    collections::HashMap,
};

use slog::Drain;

use oraide_miniyaml::{
    Lexer,
    Parser,
    Arborist,
    Tree,
    File,
    FileId,
    Files,
};

pub mod built_meta {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

fn main() {
    let decorator = slog_term::TermDecorator::new().stdout().build();
    let fmt = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_envlogger::new(fmt);
    let drain = slog_async::Async::default(drain).fuse();

    let logger = slog::Logger::root(
        drain,
        slog::o!(
            "build" => built_meta::GIT_VERSION.unwrap_or("<unknown>"),
            "version" => env!("CARGO_PKG_VERSION"),
        )
    );

    let _ = slog_envlogger::init().unwrap();
    slog_scope::scope(&logger, run);
}

fn add_file_to_files_db<'files, P: AsRef<Path>>(files: &'files mut Files, path: P) -> Result<FileId, String> {
    let path = path.as_ref();
    let path_display = path.display();

    let content = {
        let mut f = fs::File::open(path).map_err(|e| format!("Failed to open `{}`: {}", path_display, e))?;
        let mut s = String::new();
        f.read_to_string(&mut s).map_err(|e| format!("Failed to read `{}`: {}", path_display, e))?;

        s
    };

    let file_id = files.add(path_display.to_string(), content);
    Ok(file_id)
}

fn get_tree_from_file<'files>(file: &'files File) -> Result<Tree<'files>, String> {
    let lexer = Lexer::new(file);
    let tokens = lexer.collect::<Vec<_>>();

    let parser = Parser::new(tokens.into_iter());
    let nodes = parser.collect::<Vec<_>>();

    let mut arborist = Arborist::new(nodes.into_iter());
    Ok(arborist.build_tree())
}

fn run() {
    let root_dir_arg = env::args().nth(1).expect("Please provide a directory path");
    let mut files = Files::new();
    let mut map_fpath_to_fid = HashMap::new();

    let project = oraws::Project::new_from_abs_dir(root_dir_arg)
        .expect("Failed to create Project from directory");

    for (game_id, shrd_game) in &project.games {
        let manifest_path_abs = shrd_game.manifest_path_abs(&project);

        log::info!("Processing `{}` manifest at {}", game_id, manifest_path_abs.display());

        let manifest_tree = {
            let file_id = match add_file_to_files_db(&mut files, &manifest_path_abs) {
                Ok(fid) => {
                    map_fpath_to_fid.insert(manifest_path_abs.clone(), fid);

                    fid
                },
                Err(e) => {
                    log::warn!(
                        "Failed to process game manifest at {}: {}",
                        manifest_path_abs.display(),
                        e,
                    );

                    continue;
                },
            };

            let shrd_file = &files[file_id];

            match get_tree_from_file(shrd_file) {
                Ok(tree) => tree,
                Err(e) => {
                    log::warn!(
                        "Failed to process game manifest at {}: {}",
                        manifest_path_abs.display(),
                        e,
                    );

                    continue;
                },
            }
        };

        let Tree { node_ids, arena } = manifest_tree;

        let mut iter = arena.iter().zip(node_ids);

        let (_opt_shrd_rules_arena_node, opt_rules_arena_node_id) = iter.find(|(arena_node, _arena_node_id)| {
            let first_key_token = match arena_node.data.key_tokens.first() {
                Some(n) => n,
                _ => return false,
            };

            first_key_token.slice == "Rules"
        }).map_or((None, None), |(arena_node, arena_node_id)| (Some(&arena_node.data), Some(arena_node_id)));

        if let Some(rules_arena_node_id) = opt_rules_arena_node_id {
            let child_ids = rules_arena_node_id.children(&arena);

            // get the arena nodes
            let arena_nodes = child_ids.filter_map(|id| arena.get(id));

            // get the key text
            let key_slices = arena_nodes.filter_map(|arena_node| arena_node.data.key_slice(&files));

            // split on '|'
            let pipe_splits = key_slices.filter_map(|slice| {
                let mut split = slice.splitn(2, '|');
                match (split.next(), split.next()) {
                    (Some(pfx), Some(sfx)) => Some((pfx, sfx)),
                    _ => None,
                }
            });

            // get the resolved, absolute paths
            let abs_paths = pipe_splits.filter_map(|(game_id, rel_path)| {
                let shrd_game = match project.games.get(game_id) {
                    Some(g) => g,
                    _ => panic!("Game not found in gamedb"),
                };

                let game_abs_path = shrd_game.abs_path(&project);
                Some(game_abs_path.join(rel_path))
            }).collect::<Vec<_>>();

            for abs_path in abs_paths {
                log::info!("  Processing {} ...", abs_path.display());

                let file_id = match add_file_to_files_db(&mut files, &abs_path) {
                    Ok(fid) => {
                        map_fpath_to_fid.insert(abs_path.clone(), fid);

                        fid
                    },
                    Err(_e) => continue,
                };

                let shrd_file = &files[file_id];
                let tree = get_tree_from_file(shrd_file).unwrap();
                let node_count = tree.node_ids.len();
                let top_level_node_count = tree.node_ids.iter()
                    // skip the parent-less sentinel
                    .skip(1)
                    .filter_map(|&nid| tree.arena.get(nid))
                    // 'empty' nodes are technically top-level but we don't want to count them in this metric
                    .filter_map(|arena_node| if arena_node.data.is_empty() { None } else { Some(&arena_node.data.indentation_token) })
                    .filter(|shrd_indent_token| shrd_indent_token.is_none())
                    .count();

                log::info!(
                    "     created tree with {} nodes total, {} top-level nodes",
                    node_count,
                    top_level_node_count,
                );
            }
        }
    }

    for (fpath, fid) in &map_fpath_to_fid {
        log::warn!("{} -> {:?}", fpath.display(), fid);
    }
}