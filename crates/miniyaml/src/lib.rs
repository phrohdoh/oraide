// oraide - tools for OpenRA-based mod/game development
// get the source code at https://github.com/Phrohdoh/oraide
//
// copyright (c)
// - 2020 Taryn "Phrohdoh" Hill

#![deny(missing_docs)]

//! This [crate] is responsible for processing [MiniYaml] into a
//! representation which is used to perform [static analysis].
//!
//! [crate]: https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html
//! [MiniYaml]: https://www.openra.net/book/glossary.html#miniyaml
//! [static analysis]: https://en.wikipedia.org/wiki/Static_program_analysis

mod spanner;

use {
    std::{
        fmt,
    },
};

// ----- public interface ------------------------------------------------------

pub use {
    spanner::{
        span_lines_of,
        Spanner,
        SpannedLine,
    },
};

/// low-inclusive, high-exclusive span of absolute byte indices
#[derive(Copy, Clone, PartialEq)]
pub struct AbsByteIdxSpan {
    start: AbsByteIdx,
    end: AbsByteIdx,
}

/// absolute byte index
#[derive(Copy, Clone, PartialEq)]
pub struct AbsByteIdx(usize);

// ----- external trait impls --------------------------------------------------

impl fmt::Debug for AbsByteIdxSpan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "@[{}..{})", self.start.0, self.end.0)
    }
}

impl std::ops::Index<AbsByteIdxSpan> for String {
    type Output = str;
    fn index(&self, span: AbsByteIdxSpan) -> &Self::Output {
        &self[span.start.0 .. span.end.0]
    }
}

impl std::ops::Index<AbsByteIdxSpan> for str {
    type Output = Self;
    fn index(&self, span: AbsByteIdxSpan) -> &Self::Output {
        &self[span.start.0 .. span.end.0]
    }
}

impl From<AbsByteIdxSpan> for (usize, usize) {
    fn from(span: AbsByteIdxSpan) -> Self {
        (
            span.start.0,
            span.end.0,
        )
    }
}

impl From<(AbsByteIdx, AbsByteIdx)> for AbsByteIdxSpan {
    fn from((start, end): (AbsByteIdx, AbsByteIdx)) -> Self {
        Self {
            start,
            end,
        }
    }
}

impl From<(usize, usize)> for AbsByteIdxSpan {
    fn from((start, end): (usize, usize)) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }
}

impl From<usize> for AbsByteIdx {
    fn from(abs_idx: usize) -> Self {
        Self(abs_idx)
    }
}
