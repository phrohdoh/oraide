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
        Node,
        Tree,
        TextFilesCtx,
    },
};

mod queries;

/// Provides MiniYaml-parsing inputs & queries
#[salsa::query_group(ParserCtxStorage)]
pub trait ParserCtx: TextFilesCtx {
    /// Compute all of the [`Token`]s in a [`FileId`]
    ///
    /// [`Token`]: struct.Token.html
    /// [`FileId`]: struct.FileId.html
    #[salsa::invoke(queries::file_tokens)]
    fn file_tokens(&self, file_id: FileId) -> Option<Vec<Token>>;

    /// Compute all of the [`Node`]s in a [`FileId`]
    ///
    /// [`Node`]: struct.Node.html
    /// [`FileId`]: struct.FileId.html
    #[salsa::invoke(queries::file_nodes)]
    fn file_nodes(&self, file_id: FileId) -> Option<Vec<Node>>;

    /// Compute the [`Tree`] of a [`FileId`]
    ///
    /// [`Tree`]: struct.Tree.html
    /// [`FileId`]: struct.FileId.html
    #[salsa::invoke(queries::file_tree)]
    fn file_tree(&self, file_id: FileId) -> Option<Tree>;

    /// Find the top-level `Node` in `file_id` with `key`
    #[salsa::invoke(queries::top_level_node_by_key_in_file)]
    fn top_level_node_by_key_in_file(
        &self,
        file_id: FileId,
        key: String,
    ) -> Option<Node>;

    /// Compute the top-level `Node`s of a [`FileId`]
    ///
    /// This is, essentially, the file's symbol table
    ///
    /// [`FileId`]: struct.FileId.html
    #[salsa::invoke(queries::all_top_level_nodes_in_file)]
    fn all_top_level_nodes_in_file(&self, file_id: FileId) -> Option<Vec<Node>>;

    /// Compute the definitions (top-level nodes) of all files
    ///
    /// This is, essentially, a symbole table for all files
    ///
    /// # Returns
    /// For each file:
    /// - The file's ID
    /// - The file's definitions
    #[salsa::invoke(queries::top_level_nodes_in_all_files)]
    fn top_level_nodes_in_all_files(&self) -> Option<Vec<(FileId, Vec<Node>)>>;

    /// Compute the `Token` in `file_id` with a span containing `byte_index`, if any
    #[salsa::invoke(queries::token_spanning_byte_index_in_file)]
    fn token_spanning_byte_index_in_file(
        &self,
        file_id: FileId,
        byte_index: ByteIndex,
    ) -> Option<Token>;

    /// Compute the `Node` in `file_id` with a span containing `byte_index`, if any
    #[salsa::invoke(queries::node_spanning_byte_index_in_file)]
    fn node_spanning_byte_index_in_file(
        &self,
        file_id: FileId,
        byte_index: ByteIndex,
    ) -> Option<Node>;
}