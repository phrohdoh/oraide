// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{
    Token,
    TokenKind,
    FileSpan,
};

pub trait TokenCollectionExts {
    /// Get a slice of `Token`s that starts *after* leading `TokenKind::Whitespace`s
    fn skip_leading_whitespace(&self) -> &[Token<'_>];

    /// Get a span covering the entire collection of `Token`s
    ///
    /// Typically this is used to get the span of a single node (which, in practice, is an entire line)
    fn span(&self) -> Option<FileSpan>;
}

impl TokenCollectionExts for [Token<'_>] {
    fn skip_leading_whitespace(&self) -> &[Token<'_>] {
        if self.is_empty() {
            return &[];
        }

        match self.iter().position(|shrd_token| shrd_token.kind != TokenKind::Whitespace) {
            Some(idx) => &self[idx..],
            _ => &[],
        }
    }

    fn span(&self) -> Option<FileSpan> {
        if self.is_empty() {
            return None;
        }

        let first = self.first().unwrap();
        let start = first.span.start();
        let end = self.last().unwrap().span.end();

        Some(FileSpan::new(first.span.source(), start, end))
    }
}