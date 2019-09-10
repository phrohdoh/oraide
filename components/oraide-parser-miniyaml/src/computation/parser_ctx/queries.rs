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
// along with oraide.  If not, see <https://www.gnu.org/licenses/>.

use {
    oraide_span::{
        FileId,
        ByteIndex,
    },
    crate::{
        Token,
        Tokenizer,
        Node,
        Nodeizer,
        Tree,
        Treeizer,
        ParserCtx,
    },
};

pub(crate) fn file_tokens(
    db: &impl ParserCtx,
    file_id: FileId,
) -> Option<Vec<Token>> {
    let file_text = db.file_text(file_id)?;
    let mut tokenizer = Tokenizer::new(file_id, &file_text);
    tokenizer.run().into()
}

pub(crate) fn file_nodes(
    db: &impl ParserCtx,
    file_id: FileId,
) -> Option<Vec<Node>> {
    let tokens = db.file_tokens(file_id)?;
    let mut nodeizer = Nodeizer::new(tokens.into_iter());
    nodeizer.run().into()
}

pub(crate) fn file_tree(
    db: &impl ParserCtx,
    file_id: FileId,
) -> Option<Tree> {
    let nodes = db.file_nodes(file_id)?;
    let file_text = db.file_text(file_id)?;
    let mut treeizer = Treeizer::new(nodes.into_iter(), &file_text);
    treeizer.run().into()
}

pub(crate) fn top_level_node_by_key_in_file(
    db: &impl ParserCtx,
    file_id: FileId,
    key: String,
) -> Option<Node> {
    let file_text = db.file_text(file_id)?;

    db.all_top_level_nodes_in_file(file_id)?.into_iter()
        .find(|node| node.key_text(&file_text) == Some(&key))
}

pub(crate) fn all_top_level_nodes_in_file(
    db: &impl ParserCtx,
    file_id: FileId,
) -> Option<Vec<Node>> {
    let tree = db.file_tree(file_id)?;

    let top_level_nodes: Vec<_> = tree.node_ids.iter().skip(1) // skip the sentinel
        .filter_map(|arena_node_id| tree.arena.get(*arena_node_id).map(|shrd_arena_node| &shrd_arena_node.data))
        .filter(|shrd_node| shrd_node.is_top_level() && shrd_node.has_key())
        .map(|shrd_node| shrd_node.clone())
        .collect();

    top_level_nodes.into()
}

pub(crate) fn top_level_nodes_in_all_files(
    db: &impl ParserCtx,
) -> Option<Vec<(FileId, Vec<Node>)>> {
    db.all_file_ids().into_iter()
        .filter_map(|file_id| {
            let top_level_nodes = db.all_top_level_nodes_in_file(file_id)?;
            (file_id, top_level_nodes).into()
        })
        .collect::<Vec<_>>()
        .into()
}

pub(crate) fn token_spanning_byte_index_in_file(
    db: &impl ParserCtx,
    file_id: FileId,
    byte_index: ByteIndex,
) -> Option<Token> {
    let tokens = db.file_tokens(file_id)?;
    tokens.into_iter().find(|token| token.span.contains(byte_index))
}

pub(crate) fn node_spanning_byte_index_in_file(
    db: &impl ParserCtx,
    file_id: FileId,
    byte_index: ByteIndex,
) -> Option<Node> {
    let nodes = db.file_nodes(file_id)?;
    nodes.into_iter()
        .find(|node| node.span().map(|span| span.contains(byte_index)).unwrap_or(false))
}