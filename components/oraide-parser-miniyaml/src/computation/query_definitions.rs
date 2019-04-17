use oraide_span::{
    FileId,
    FileSpan,
};

use crate::{
    Token,
    Tokenizer,
    Node,
    Nodeizer,
    Tree,
    Treeizer,
    ParserCtx,
};

pub(crate) fn file_tokens(db: &impl ParserCtx, file_id: FileId) -> Vec<Token> {
    let text = db.file_text(file_id);
    let mut tokenizer = Tokenizer::new(file_id, &text);
    tokenizer.run()
}

pub(crate) fn file_nodes(db: &impl ParserCtx, file_id: FileId) -> Vec<Node> {
    let tokens = db.file_tokens(file_id);
    let mut nodeizer = Nodeizer::new(tokens.into_iter());
    nodeizer.run()
}

pub(crate) fn file_tree(db: &impl ParserCtx, file_id: FileId) -> Tree {
    let text = db.file_text(file_id);
    let nodes = db.file_nodes(file_id);
    let mut treeizer = Treeizer::new(nodes.into_iter(), &text);
    treeizer.run()
}

pub(crate) fn file_definitions(db: &impl ParserCtx, file_id: FileId) -> Vec<Node> {
    let tree = db.file_tree(file_id);

    let top_level_nodes = tree.node_ids.iter().skip(1) // skip the sentinel
        .filter_map(|nid| tree.arena.get(*nid).map(|an| (*nid, &an.data)))
        .filter(|(_nid, shrd_node)| shrd_node.is_top_level() && shrd_node.has_key())
        .map(|(_nid, shrd_node)| shrd_node.clone())
        .collect::<Vec<_>>();

    top_level_nodes
}

pub(crate) fn all_definitions(db: &impl ParserCtx) -> Vec<(FileId, Vec<Node>)> {
    db.all_file_ids()
        .into_iter()
        .map(|file_id| (file_id, db.file_definitions(file_id)))
        .collect()
}

/// Find a definition with name `def_name` and return its span.
pub(crate) fn file_definition_span(db: &impl ParserCtx, file_id: FileId, def_name: String) -> Option<FileSpan> {
    let text = db.file_text(file_id);
    let defs = db.file_definitions(file_id);

    for def in defs {
        let key_text = match def.key_text(&text) {
            Some(text) => text,
            _ => continue,
        };

        if key_text == &def_name {
            return def.key_span();
        }
    }

    None
}