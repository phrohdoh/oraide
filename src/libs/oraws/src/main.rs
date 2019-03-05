use std::{env, fs, io::Read as _};
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
        let arena = {
            let lexer = oraml::Lexer::new(&files[manifest_file_id]);
            let tokens = lexer.collect::<Vec<_>>();

            let parser = oraml::Parser::new(manifest_file_id, tokens.into_iter());
            let nodes = parser.collect::<Vec<_>>();

            let mut arborist = oraml::Arborist::new(nodes.into_iter());
            arborist.build_tree()
        };

        let metadata_opt_node_ref = arena.iter().find(|&arena_node| {
            let first_key_token = match arena_node.data.key_tokens.first() {
                Some(n) => n,
                _ => return false,
            };

            first_key_token.slice == "Metadata"
        }).map(|arena_node| &arena_node.data);

        if let Some(metadata_node_ref) = metadata_opt_node_ref {
            log::error!("{}: {:?}", game.id(), metadata_node_ref);
        }
    }
}