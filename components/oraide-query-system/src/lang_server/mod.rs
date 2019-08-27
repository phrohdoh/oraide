use std::{
    io,
    path::{
        PathBuf,
    },
};

use derive_more::{
    Display,
    From,
};

use url::Url;

use oraide_parser_miniyaml::{
    ParserCtx,
};

use oraide_span::{
    ByteIndex,
    FileId,
};

use oraide_actor::{
    Position,
};

use crate::{
    query_definitions,
};

pub(crate) mod types;

/// A newtype over [`String`] used for [`LangServerCtx::hover`]
///
/// [`String`]: https://doc.rust-lang.org/nightly/alloc/string/struct.String.html
/// [`LangServerCtx::hover`]: trait.LangServerCtx.html#method.hover
#[derive(Debug, Clone, Display, From, PartialEq, Eq, Hash)]
pub struct Markdown(pub String);

// TODO: Why does https://github.com/lark-exploration/lark/blob/f875d24011d7c362bed2d1ed73900ef0f2109445/components/lark-query-system/src/ls_ops.rs#L32
//       not have a `salsa::query_group` attr?
#[salsa::query_group(LangServerCtxStorage)]
pub trait LangServerCtx: ParserCtx {
    #[salsa::input]
    fn workspace_root(&self) -> Option<PathBuf>;

    #[salsa::invoke(query_definitions::type_data)]
    fn type_data(&self) -> Option<Vec<types::TraitDetail>>;

    #[salsa::invoke(query_definitions::type_data_json_file_path)]
    fn type_data_json_file_path(&self) -> Option<PathBuf>;

    #[salsa::invoke(query_definitions::doc_lines_for_trait)]
    fn doc_lines_for_trait(&self, trait_name: String) -> Option<Vec<String>>;

	/// Compute the [`ByteIndex`] that a [`Position`] in `file_id` maps to
    ///
    /// # Returns
    /// - `None` if `pos.line` is greater than (the number of lines in `file_id` - 1)
    ///
    /// [`Position`]: struct.Position.html
    /// [`ByteIndex`]: ../oraide-span/byte/struct.ByteIndex.html
    #[salsa::invoke(query_definitions::position_to_byte_index)]
    fn position_to_byte_index(&self, file_id: FileId, pos: Position) -> Option<ByteIndex>;

    #[salsa::invoke(query_definitions::byte_index_to_position)]
    fn byte_index_to_position(&self, file_id: FileId, byte_index: ByteIndex) -> Option<Position>;

    /// Compute the hover data for a [`Position`] in `file_name`
    ///
    /// [`Position`]: struct.Position.html
    #[salsa::invoke(query_definitions::hover_with_file_name)]
    fn hover_with_file_name(&self, file_name: String, pos: Position) -> Option<Markdown>;

    /// Compute the hover data for a [`Position`] in `file_id`
    ///
    /// [`Position`]: struct.Position.html
    #[salsa::invoke(query_definitions::hover_with_file_id)]
    fn hover_with_file_id(&self, file_id: FileId, pos: Position) -> Option<Markdown>;

    /// Compute the definition of a symbol at `position` in `file_name`
    #[salsa::invoke(query_definitions::definition_with_file_name)]
    fn definition_with_file_name(&self, file_name: String, pos: Position) -> Option<(Url, Position, Position)>;

    /// Compute the definition of a symbol at `position` in file with id `file_id`
    #[salsa::invoke(query_definitions::definition_with_file_id)]
    fn definition_with_file_id(&self, file_id: FileId, pos: Position) -> Option<(Url, Position, Position)>;
}