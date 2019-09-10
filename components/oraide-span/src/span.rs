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

        assert!(start.to_usize() <= end_exclusive.to_usize());

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

    /// Determine whether this span contains `byte_index`
    ///
    /// If `byte_index` is equal to this span's `end_exclusive` it is considered
    /// not contained within this span.
    pub fn contains(&self, byte_index: impl Into<ByteIndex>) -> bool {
        let byte_index = byte_index.into();
        self.start <= byte_index && byte_index < self.end_exclusive
    }

    /// Determine whether this span contains `span`
    pub fn contains_span(&self, span: Self) -> bool {
        if !self.contains(span.start().to_usize()) {
            return false;
        }

        if !self.contains(span.end_exclusive().to_usize() - 1) {
            return false;
        }

        true
    }

    /// Get the text slice of this span
    ///
    /// # Returns
    /// - `None` if either of the components of this span lie outside of
    ///   `source_text` or are not on valid character boundaries
    /// - `Some(_)` otherwise
    pub fn text<'text>(&self, source_text: &'text str) -> Option<&'text str> {
        let start = self.start.to_usize();
        if !source_text.is_char_boundary(start) {
            return None;
        }

        let end_exclusive = self.end_exclusive.to_usize();
        if !source_text.is_char_boundary(end_exclusive) {
            return None;
        }

        Some(&source_text[start..end_exclusive])
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    /// Compute the `ByteIndex` of the `n`-th (1-based) `ch` in `s`
    ///
    /// # Example
    /// ```rust
    /// let idx_of_2nd_n = byte_index_of_nth_char_in_str(2, 'n', "Name: McKenna");
    /// ```
    fn byte_index_of_nth_char_in_str(n: usize, ch: char, s: &str) -> ByteIndex {
        assert!(n > 0, "n={}", n);
        assert!(n < s.len(), "n={} < s.len()={}", n, s.len());

        let idx = s
            .match_indices(ch)
            .nth(n - 1) // `nth` is 0-based (https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.nth)
            .map(|(idx, _)| idx)
            .expect(&format!(
                "TEST LOGIC ERROR: {}({}, {}, {:?})",
                stringify!(byte_index_of_nth_char_in_str),
                n, ch, s
            ));

        ByteIndex::from(idx)
    }

    #[test]
    fn contains() {
        let span = Span::from_str(FileId(0), "Hello:\n\tWorld: Terra\n\t\tChickens:\n");
        assert!(span.contains(ByteIndex::from(0)));

        let end = ByteIndex::from(span.end_exclusive.to_usize() - 1);
        assert!(span.contains(end));
    }

    #[test]
    fn contains_does_not_include_end_exclusive() {
        let span = Span::from_str(FileId(0), "Hello: World");
        let end_exclusive = span.end_exclusive();
        assert!(!span.contains(end_exclusive));
    }

    #[test]
    fn contains_does_not_include_out_of_bounds() {
        let span = Span::from_str(FileId(0), "Hello: World");
        let out_of_bounds = ByteIndex::from(span.end_exclusive().to_usize() + 1);
        assert!(!span.contains(out_of_bounds));
    }

    #[test]
    fn text() {
        // Arrange
        let input = "foo bar baz";
        let span = {
            let start = byte_index_of_nth_char_in_str(1, 'b', input);
            let end_exclusive = byte_index_of_nth_char_in_str(1, 'r', input) + 1.into();

            FileSpan::new(FileId(0), start, end_exclusive)
        };

        // Act
        let opt_text = span.text(input);

        // Assert
        assert_eq!(opt_text, Some("bar"));
    }

    #[test]
    fn text_invalid_start() {
        // Arrange
        let input = "ÿ";
        let span = FileSpan::new(FileId(0), 0, 1); // `[0, 2)` would be valid

        // Act
        let opt_text = span.text(input);

        // Assert
        assert_eq!(opt_text, None);
    }

    #[test]
    fn text_invalid_end_exclusive() {
        // Arrange
        let input = "ÿ";
        let invalid_span = FileSpan::new(FileId(0), 1, 2);

        // Act
        let opt_text = invalid_span.text(input);

        // Assert
        assert_eq!(opt_text, None);
    }
}