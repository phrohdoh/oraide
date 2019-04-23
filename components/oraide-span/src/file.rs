// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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