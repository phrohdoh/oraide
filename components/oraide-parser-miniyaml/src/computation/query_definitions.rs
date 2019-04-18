use oraide_span::{
    FileId,
    FileSpan,
    ByteIndex,
    Location,
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

pub(crate) fn line_offsets(db: &impl ParserCtx, file_id: FileId) -> Vec<usize> {
    let text = &db.file_text(file_id);
    let mut acc = 0;

    text.lines()
        .map(|line_text| {
            let line_start = acc;
            acc += line_text.len();

            if text[acc..].starts_with("\r\n") {
                acc += 2;
            } else if text[acc..].starts_with("\n") {
                acc += 1;
            }

            line_start
        })
        .chain(std::iter::once(text.len()))
        .collect()
}

/// Convert a [`ByteIndex`] into a [`Location`] using [`line_offsets`] to
/// quickly find the byte index the line start locations
///
/// [`ByteIndex`]: struct.ByteIndex.html
/// [`Location`]: struct.Location.html
/// [`line_offsets`]: #fn.line_offsets
pub(crate) fn location(db: &impl ParserCtx, file_id: FileId, index: ByteIndex) -> Location {
    let line_offsets = db.line_offsets(file_id);

    match line_offsets.binary_search(&index.to_usize()) {
        Ok(line_idx) => {
            // Found the start of the line directly
            Location::new(line_idx + 1, 1)
        },
        Err(next_line_num) => {
            let line_idx = next_line_num - 1;

            // Found something in the middle
            let line_start_idx = line_offsets[line_idx];

            // Count utf-8 chars to determine column
            let text = &db.file_text(file_id);
            let column = text[line_start_idx..index.to_usize()].chars().count();

            Location::new(next_line_num, column)
        },
    }
}

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