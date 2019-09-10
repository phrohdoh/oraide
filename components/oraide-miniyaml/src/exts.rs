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