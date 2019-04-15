use std::{
    env,
    io::{
        Read as _,
    },
    fs::{
        File,
    },
};

use oraide_span::{
    FileId,
};

use oraide_parser_miniyaml::{
    Tokenizer,
    Nodeizer,
    Treeizer,
    TokenCollectionExts as _,
};

fn main() -> Result<(), String> {
    run(&mut env::args().skip(1))
}

type CliArgs = dyn Iterator<Item = String>;

fn run(args: &mut CliArgs) -> Result<(), String> {
    let file_path = args.next()
        .ok_or_else(|| "Please provide a file path".to_string())?;

    let file_contents = {
        let mut file = File::open(&file_path)
            .map_err(|e| format!("Failed to open `{}`: {}", file_path, e))?;

        let mut s = String::new();

        file.read_to_string(&mut s)
            .map_err(|e| format!("Failed to read `{}`: {}", file_path, e))?;

        s
    };

    let file_id = FileId(0);

    let mut tokenizer = Tokenizer::new(file_id, &file_contents);
    let tokens = tokenizer.run();

    let mut nodeizer = Nodeizer::new(tokens.into_iter());
    let nodes = nodeizer.run();

    let mut treeizer = Treeizer::new(nodes.into_iter(), &file_contents);
    let tree = treeizer.run();

    let top_level_nodes = tree.node_ids.iter().skip(1) // skip the sentinel
        .filter_map(|nid| tree.arena.get(*nid).map(|an| (*nid, &an.data)))
        .filter(|(_nid, shrd_node)| shrd_node.is_top_level() && shrd_node.has_key());

    let top_level_slices = top_level_nodes
        .filter_map(|(_nid, shrd_node)| shrd_node.key_tokens.span())
        .map(|span| {
            let start = span.start().to_usize();
            let end_exclusive = span.end_exclusive().to_usize();
            &file_contents[start..end_exclusive]
        })
        .collect::<Vec<_>>();

    println!("{} has {} top-level node(s):", file_path, top_level_slices.len());

    for name in top_level_slices {
        println!(" - {}", name);
    }

    Ok(())
}