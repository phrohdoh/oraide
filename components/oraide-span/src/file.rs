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

use crate::{
    Span,
};

/// A handle that points to a file in a file database
/// 
/// # Example
/// ```rust
/// # use oraide_span::{FileId};
/// let file_id = FileId(0);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub usize);

/// A `Span` with a `FileId` source
/// 
/// # Example
/// ```rust
/// # use oraide_span::{FileSpan,FileId};
/// let file_span = FileSpan::new(FileId(0), 17, 24);
/// ```
pub type FileSpan = Span<FileId>;