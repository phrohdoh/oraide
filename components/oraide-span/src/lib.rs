use std::{
    fmt,
    ops,
};

/// Byte index into a string
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteIndex(pub usize);

impl ByteIndex {
    pub fn to_usize(self) -> usize {
        self.0
    }
}

impl From<usize> for ByteIndex {
    fn from(src: usize) -> ByteIndex {
        ByteIndex(src)
    }
}

impl ops::Add<ByteCount> for ByteIndex {
    type Output = ByteIndex;

    fn add(self, other: ByteCount) -> ByteIndex {
        ByteIndex::from(self.to_usize() + other.to_usize())
    }
}

impl ops::AddAssign<ByteCount> for ByteIndex {
    fn add_assign(&mut self, other: ByteCount) {
        self.0 += other.to_usize();
    }
}

impl ops::Sub<ByteIndex> for ByteIndex {
    type Output = ByteCount;

    fn sub(self, other: ByteIndex) -> ByteCount {
        ByteCount::from(self.to_usize() - other.to_usize())
    }
}

/// Byte count for a given string or character in some encoding
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteCount(pub usize);

impl ByteCount {
    /// ```rust
    /// # use oraide_span::ByteCount;
    /// let ch = 'ç';
    /// let byte_count = ByteCount::from_char_len_utf8(ch);
    /// 
    /// assert_eq!(byte_count, ByteCount(2));
    /// ```
    pub fn from_char_len_utf8(ch: char) -> ByteCount {
        ByteCount::from(ch.len_utf8())
    }

    /// ```rust
    /// # use oraide_span::ByteCount;
    /// let ch = 'ç';
    /// let byte_count = ByteCount::from_char_len_utf16(ch);
    /// 
    /// assert_eq!(byte_count, ByteCount(1));
    /// ```
    pub fn from_char_len_utf16(ch: char) -> ByteCount {
        ByteCount::from(ch.len_utf16())
    }

    /// ```rust
    /// # use oraide_span::ByteCount;
    /// let s = "true";
    /// let byte_count = ByteCount::from_str_len_utf8(s);
    /// 
    /// assert_eq!(byte_count, ByteCount(4));
    /// #
    /// # // exclude the remainder from the docs for the sake of
    /// # // brevity, but include them in the tests
    /// # assert_eq!(ByteCount::from_str_len_utf8("a"), ByteCount(1));
    /// # assert_eq!(ByteCount::from_str_len_utf8("å"), ByteCount(2));
    /// # assert_eq!(ByteCount::from_str_len_utf8("“"), ByteCount(3));
    /// ```
    pub fn from_str_len_utf8(s: &str) -> ByteCount {
        ByteCount::from(s.len())
    }

    pub fn to_usize(self) -> usize {
        self.0
    }
}

impl ops::Add<ByteCount> for ByteCount {
    type Output = ByteCount;

    fn add(self, other: ByteCount) -> ByteCount {
        ByteCount::from(self.to_usize() + other.to_usize())
    }
}

impl ops::AddAssign<ByteCount> for ByteCount {
    fn add_assign(&mut self, other: ByteCount) {
        self.0 += other.to_usize();
    }
}

impl From<usize> for ByteCount {
    fn from(src: usize) -> ByteCount {
        ByteCount(src)
    }
}

/// A `Span` with a `FileId` source
/// 
/// # Example
/// ```rust
/// # use oraide_span::{FileSpan,FileId};
/// let file_span = FileSpan::new(FileId(0), 17, 24);
/// ```
pub type FileSpan = Span<FileId>;

/// A handle that points to a file in a file database
/// 
/// # Example
/// ```rust
/// # use oraide_span::{FileId};
/// let file_id = FileId(0);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub usize);

/// Used to track spans in text documents
/// 
/// ```rust
/// # use oraide_span::{Span,FileId};
/// let span = Span::new(FileId(0), 0, 2); // (0, 2]
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span<TSource> {
    source: TSource,
    start: ByteIndex,
    end_exclusive: ByteIndex,
}

impl<TSource: Copy + fmt::Debug> Span<TSource> {
    pub fn new(source: TSource, start: impl Into<ByteIndex>, end_exclusive: impl Into<ByteIndex>) -> Span<TSource> {
        let start = start.into();
        let end_exclusive = end_exclusive.into();

        Self {
            source,
            start,
            end_exclusive,
        }
    }

    /// Gives an empty span at the start of a source
    pub fn initial(source: TSource) -> Span<TSource> {
        Span::new(source, 0, 0)
    }

    /// ```rust
    /// # use oraide_span::{Span,FileId,ByteCount};
    /// let source = FileId(0);
    /// 
    /// let span = Span::from_str(source, "hello, friends!");
    /// assert_eq!(span.len(), ByteCount(15));
    /// #
    /// # // exclude the remainder from the docs for the sake of
    /// # // brevity, but include them in the tests
    /// # let span = Span::from_str(source, "مرحبا أيها الأصدقاء!");
    /// # assert_eq!(span.len(), ByteCount(37));
    /// #
    /// # let span = Span::from_str(source, "皆さん、こんにちは！");
    /// # assert_eq!(span.len(), ByteCount(30));
    /// #
    /// # let span = Span::from_str(source, "\n");
    /// # assert_eq!(span.len(), ByteCount(1));
    /// #
    /// # let span = Span::from_str(source, "\r\n");
    /// # assert_eq!(span.len(), ByteCount(2));
    /// ```
    pub fn from_str(source: TSource, s: &str) -> Span<TSource> {
        Span::new(source, 0, s.len())
    }

    /// ```rust
    /// # use oraide_span::{Span,FileId};
    /// let a = Span::new(FileId(0), 0, 11);
    /// let b = a.with_source(FileId(1));
    /// # assert_eq!(a.source(), FileId(0));
    /// assert_eq!(b.source(), FileId(1));
    /// ```
    pub fn with_source<TNewSource: Copy + fmt::Debug>(&self, source: TNewSource) -> Span<TNewSource> {
        Span::new(source, self.start(), self.end_exclusive())
    }

    /// ```rust
    /// # use oraide_span::{Span,FileId,ByteIndex};
    /// let a = Span::new(FileId(0), 0, 11);
    /// let b = a.with_start(5);
    /// # assert_eq!(a.start(), ByteIndex(0));
    /// assert_eq!(b.start(), ByteIndex(5));
    /// ```
    pub fn with_start(&self, start: impl Into<ByteIndex>) -> Span<TSource> {
        Span::new(self.source(), start, self.end_exclusive())
    }

    /// ```rust
    /// # use oraide_span::{Span,FileId,ByteIndex};
    /// let a = Span::new(FileId(0), 0, 11);
    /// let b = a.with_end_exclusive(47);
    /// # assert_eq!(a.end_exclusive(), ByteIndex(11));
    /// assert_eq!(b.end_exclusive(), ByteIndex(47));
    /// ```
    pub fn with_end_exclusive(&self, end_exclusive: impl Into<ByteIndex>) -> Span<TSource> {
        Span::new(self.source(), self.start(), end_exclusive)
    }

    pub fn source(&self) -> TSource {
        self.source
    }

    pub fn start(&self) -> ByteIndex {
        self.start
    }

    pub fn end_exclusive(&self) -> ByteIndex {
        self.end_exclusive
    }

    pub fn len(&self) -> ByteCount {
        self.end_exclusive() - self.start()
    }
}