// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    fmt,
};

use crate::{
    ByteIndex,
    ByteCount,
};

/// Used to track spans in text documents
///
/// A Span, `span`, can be expressed in _interval notation_ as
/// `[span.start, span.end_exclusive)`
/// 
/// ```rust
/// # use oraide_span::{Span,FileId};
/// let span = Span::new(FileId(0), 0, 2); // [0, 2)
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