use derive_more::{
    Display,
    From,
};

pub use languageserver_types::{
    Position as LsPos,
};

use oraide_parser_miniyaml::{
    ParserCtx,
};

use oraide_span::{
    ByteIndex,
    FileId,
};

/// A newtype over [`String`] used for [`LangServerCtx::hover`]
///
/// [`String`]: https://doc.rust-lang.org/nightly/alloc/string/struct.String.html
/// [`LangServerCtx::hover`]: trait.LangServerCtx.html#method.hover
#[derive(Debug, Clone, Display, From)]
pub struct Markdown(pub String);

// TODO: Why does https://github.com/lark-exploration/lark/blob/f875d24011d7c362bed2d1ed73900ef0f2109445/components/lark-query-system/src/ls_ops.rs#L32
//       not have a `salsa::query_group` attr?
pub trait LangServerCtx: ParserCtx {
	/// Compute the [`ByteIndex`] that a [`LsPos`] in `file_id` maps to
    ///
    /// # Returns
    /// - `None` if `pos.line` is greater than (the number of lines in `file_id` - 1)
    ///
    /// [`LsPos`]: ../languageserver_types/struct.Position.html
    /// [`ByteIndex`]: ../oraide-span/byte/struct.ByteIndex.html
    fn position_to_byte_index(&self, file_id: FileId, pos: LsPos) -> Option<ByteIndex> {
        let line_idx = pos.line as usize;
        let char_idx = pos.character as usize;
        let line_offsets = self.line_offsets(file_id);

        let line_num = line_idx + 1;
        if line_offsets.len() < line_num {
            return None;
        }

        let line_start = line_offsets[line_idx];
        Some(ByteIndex::from(line_start + char_idx))
    }

    /// Compute the hover data for a [`LsPos`] in `file_name`
    ///
    /// [`LsPos`]: ../languageserver_types/struct.Position.html
    fn hover_with_file_name(&self, file_name: &str, pos: LsPos) -> Option<Markdown> {
        let file_id = match self.file_name_to_file_id(file_name.to_owned()) {
            Some(fid) => fid,
            _ => {
                eprintln!("No file id found for file name `{}`", file_name);
                return None;
            },
        };

        self.hover_with_file_id(file_id, pos)
    }

    /// Compute the hover data for a [`LsPos`] in `file_id`
    ///
    /// [`LsPos`]: ../languageserver_types/struct.Position.html
    fn hover_with_file_id(&self, file_id: FileId, pos: LsPos) -> Option<Markdown> {
        let file_text = self.file_text(file_id);
        let byte_index = self.position_to_byte_index(file_id, pos)?;
        let token = self.token_spanning_byte_index(file_id, byte_index)?;
        let token_text = token.text(&file_text)?;

        // Returning an empty string to the client isn't helpful, so return a
        // `None` here that we'll convert into something indicating 'no results'
        // in `QuerySystem::process_message`.
        if token_text.trim().is_empty() {
            return None;
        }

        // NEXT: See https://github.com/Phrohdoh/OpenRA/tree/oraide-util-extract-type-data
        // TODO: Lookup in cached documentation storage
        //       Currently this just spits back the text of the token
        return Some(token_text.to_owned().into());
    }
}

impl LangServerCtx for crate::Database {}