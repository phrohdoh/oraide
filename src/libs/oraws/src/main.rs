// invoke like so:
// cargo run -- ~/src/games/openra/engine/

use std::{env, fs, io::Read as _};
use slog::Drain;

use oraml::Tree;

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

fn run() {
    let root_dir_arg = env::args().nth(1).expect("Please provide a directory path");
    let mut files = oraml::Files::new();

    let project = oraws::Project::new_from_abs_dir(root_dir_arg)
        .expect("Failed to create Project from directory");

    for (game_id, shrd_game) in &project.games {
        let game_abs_path = shrd_game.abs_path(&project);

        let manifest_path_abs = shrd_game.manifest_path_abs(&project);

        let manifest_content = {
            let mut s = String::new();
            let mut f = match fs::File::open(&manifest_path_abs) {
                Ok(f) => f,
                Err(_e) => {
                    log::warn!(
                        "Failed to open game manifest `{}`, does `{}` have a root-level manifest file?",
                        manifest_path_abs.display(),
                        game_id,
                    );

                    continue;
                },
            };

            f.read_to_string(&mut s).expect(&format!(
                "Failed to read file `{}`",
                manifest_path_abs.display(),
            ));
            s
        };

        // TODO: Wrap all this up in `oraml`
        let manifest_file_id = files.add(format!("{}", manifest_path_abs.display()), manifest_content);
        let Tree { node_ids, arena } = {
            let lexer = oraml::Lexer::new(&files[manifest_file_id]);
            let tokens = lexer.collect::<Vec<_>>();

            let parser = oraml::Parser::new(manifest_file_id, tokens.into_iter());
            let nodes = parser.collect::<Vec<_>>();

            let mut arborist = oraml::Arborist::new(nodes.into_iter());
            arborist.build_tree()
        };

        let mut iter = arena.iter().zip(node_ids);

        let (_opt_ref_rules_arena_node, opt_rules_arena_node_id) = iter.find(|(arena_node, _arena_node_id)| {
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

            log::info!("-- {} at {} --", game_id, game_abs_path.display());
            for abs_path in abs_paths {
                let abs_path_display = format!("{}", abs_path.display());
                log::info!("  TODO {} ...", abs_path_display);

                //files.add(abs_path_display, {
                //    let mut f = fs::File::open(abs_path).unwrap();
                //    let mut s = String::new();
                //    let _ = f.read_to_string(&mut s).unwrap();
                //    s
                //});
            }
        }
    }
}