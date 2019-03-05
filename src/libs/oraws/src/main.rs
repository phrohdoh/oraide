use std::{env, fs, io::Read as _};
use slog::Drain;
use oraml::TokenCollectionExts as _;

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

        let (_opt_ref_metadata_arena_node, opt_metadata_arena_node_id) = iter.find(|(arena_node, _arena_node_id)| {
            let first_key_token = match arena_node.data.key_tokens.first() {
                Some(n) => n,
                _ => return false,
            };

            first_key_token.slice == "Metadata"
        }).map_or((None, None), |(arena_node, arena_node_id)| (Some(&arena_node.data), Some(arena_node_id)));

        if let Some(metadata_arena_node_id) = opt_metadata_arena_node_id {
            let mut child_ids = metadata_arena_node_id.children(&arena);

            let title_node_opt = child_ids
                .find(|&id| arena.get(id).map_or(false, |arena_node| arena_node.data.key_tokens.first().map_or(false, |token| token.slice == "Title")))
                .and_then(|title_node_id| arena.get(title_node_id).map(|arena_node| &arena_node.data));

            if let Some(title_node_ref) = title_node_opt {
                let value_string = itertools::join(
                    title_node_ref.value_tokens
                        .skip_leading_whitespace()
                        .iter()
                        .map(|token_ref| token_ref.slice),
                    ""
                );

                println!("{}: {}", game.id(), value_string.trim());
            }
        }
    }
}