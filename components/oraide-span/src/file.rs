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