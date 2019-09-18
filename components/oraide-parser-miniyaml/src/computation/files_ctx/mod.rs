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
        FileSpan,
        ByteIndex,
        Location,
    },
    oraide_actor::{
        Position,
    },
};

mod queries;

#[salsa::query_group(FilesCtxStorage)]
pub trait FilesCtx: salsa::Database {
    /// Path of the file that was assigned the given [`FileId`]
    ///
    /// [`FileId`]: ../oraide-span/struct.FileId.html
    #[salsa::input]
    fn file_path(
        &self,
        file_id: FileId,
    ) -> Option<String>;

    /// All of the tracked [`FileId`]s
    ///
    /// [`FileId`]: ../oraide-span/struct.FileId.html
    #[salsa::input]
    fn all_file_ids(
        &self
    ) -> Vec<FileId>;

    /// Find the [`FileId`] associated with `file_path`, if one exists
    ///
    /// [`FileId`]: ../oraide-span/struct.FileId.html
    #[salsa::invoke(queries::file_id_of_file_path)]
    fn file_id_of_file_path(
        &self,
        file_path: String,
    ) -> Option<FileId>;
}

pub trait FilesCtxExt: FilesCtx {
    fn init_empty_file_ids(&mut self) {
        self.set_all_file_ids(Default::default());
    }
}

#[salsa::query_group(TextFilesCtxStorage)]
pub trait TextFilesCtx: FilesCtx {
    /// Text of the file that was assigned a given [`FileId`]
    ///
    /// [`FileId`]: struct.FileId.html
    #[salsa::input]
    fn file_text(
        &self,
        file_id: FileId,
    ) -> Option<String>;

    /// Compute all line start offsets in byte indices
    #[salsa::invoke(queries::line_start_offsets)]
    fn line_start_offsets(
        &self,
        file_id: FileId,
    ) -> Option<Vec<usize>>;

    #[salsa::invoke(queries::convert_file_span_to_2_positions)]
    fn convert_file_span_to_2_positions(
        &self,
        span: FileSpan,
    ) -> Option<(Position, Position)>;

    /// Convert a [`ByteIndex`] in `file_id` into a [`Location`]
    ///
    /// [`ByteIndex`]: ../oraide-span/struct.ByteIndex.html
    /// [`Location`]: ../oraide-span/struct.Location.html
    #[salsa::invoke(queries::convert_byte_index_to_location)]
    fn convert_byte_index_to_location(
        &self,
        file_id: FileId,
        byte_index: ByteIndex,
    ) -> Option<Location>;

    /// Convert a [`Position`] into a [`ByteIndex`] in `file_id`
    ///
    /// # Returns
    /// - `None` if `pos.line` is greater than or equal to <line count in `file_id`>
    ///
    /// [`Position`]: struct.Position.html
    /// [`ByteIndex`]: ../oraide-span/byte/struct.ByteIndex.html
    #[salsa::invoke(queries::convert_position_to_byte_index)]
    fn convert_position_to_byte_index(
        &self,
        file_id: FileId,
        position: Position,
    ) -> Option<ByteIndex>;
}

pub trait TextFilesCtxExt: TextFilesCtx {
    /// Add a text file to an [`OraideDatabase`]
    ///
    /// See [`OraideDatabase`]'s docs for a code example.
    ///
    /// # Returns
    /// A newly-created [`FileId`] that uniquely represents this file in a
    /// [`OraideDatabase`]
    ///
    /// [`FileId`]: struct.FileId.html
    /// [`OraideDatabase`]: ../oraide_query_system/struct.OraideDatabase.html
    fn add_text_file(
        &mut self,
        file_path: impl Into<String>,
        file_text: impl Into<String>,
    ) -> FileId {
        let file_path = file_path.into();
        let file_text = file_text.into();

        let mut all_file_ids = self.all_file_ids();
        let file_id = FileId(all_file_ids.len());
        all_file_ids.extend(Some(file_id));

        self.set_file_path(file_id, file_path.into());
        self.set_all_file_ids(all_file_ids);
        self.set_file_text(file_id, file_text.into());

        file_id
    }
}