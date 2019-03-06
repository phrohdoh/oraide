// invoke like so:
// cargo run -- ~/src/games/openra/engine/

use std::{env, fs, io::Read as _, path::Path};
use slog::Drain;

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

    let project = oraws::Project::new_from_abs_dir(root_dir_arg)
        .expect("Failed to create Project from directory");

    let mut files = oraml::Files::new();

    for game in project.games() {
        let game_id = game.id();
        let game_abs_path = game.abs_path(&project);

        let manifest_path_abs = game.manifest_path_abs(&project);

        let manifest_content = {
            let mut s = String::new();
            let mut f = fs::File::open(&manifest_path_abs).expect(&format!(
                "Failed to open file `{}`",
                manifest_path_abs.display(),
            ));

            f.read_to_string(&mut s).expect(&format!(
                "Failed to read file `{}`",
                manifest_path_abs.display(),
            ));
            s
        };

        // TODO: Wrap all this up in `oraml`
        let manifest_file_id = files.add(format!("{}", manifest_path_abs.display()), manifest_content);
        let (all_node_ids, arena) = {
            let lexer = oraml::Lexer::new(&files[manifest_file_id]);
            let tokens = lexer.collect::<Vec<_>>();

            let parser = oraml::Parser::new(manifest_file_id, tokens.into_iter());
            let nodes = parser.collect::<Vec<_>>();

            let mut arborist = oraml::Arborist::new(nodes.into_iter());
            arborist.build_tree()
        };

        let mut iter = arena.iter().zip(all_node_ids);

        let (_opt_ref_rules_arena_node, opt_rules_arena_node_id) = iter.find(|(arena_node, _arena_node_id)| {
            let first_key_token = match arena_node.data.key_tokens.first() {
                Some(n) => n,
                _ => return false,
            };

            first_key_token.slice == "Rules"
        }).map_or((None, None), |(arena_node, arena_node_id)| (Some(&arena_node.data), Some(arena_node_id)));

        if let Some(rules_arena_node_id) = opt_rules_arena_node_id {
            let child_ids = rules_arena_node_id.children(&arena);

            let game_id_bar_prefix = format!("{}|", game_id);

            let arena_nodes = child_ids.filter_map(|id| arena.get(id));
            let key_slices = arena_nodes.filter_map(|arena_node| arena_node.data.key_slice(&files));
            let filtered_key_slices = key_slices.filter(|&slice| slice.starts_with(&game_id_bar_prefix));
            let rel_slices = filtered_key_slices.map(|slice| &slice[game_id_bar_prefix.len()..]);
            let rel_paths = rel_slices.map(|slice| Path::new(slice));
            let rel_paths = rel_paths.collect::<Vec<_>>();

            println!("-- {} at {} --", game_id, game_abs_path.display());
            for rel_path in rel_paths {
                let abs_path = game_abs_path.join(rel_path);
                println!("TODO: Process {}", abs_path.display());
            }
        }
    }
}