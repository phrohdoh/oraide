use std::fmt;

use mltt_span::{
    FileSpan,
};

/// A tag that makes it easier to store what type of token this is
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Error,

    Whitespace,
    Comment,

    Keyword,
    Symbol,
    Identifier,
    IntLiteral,
    FloatLiteral,

    Tilde,
    Bang,
    At,
    Caret,
    Colon,
    LogicalOr,
    LogicalAnd,

    // TODO: Consider an explicit Eol TokenKind
  //Eol,
}

/// A token in the source file, to be emitted by a `Lexer` instance
#[derive(Clone, PartialEq, Eq)]
pub struct Token<'file> {
    /// The token kind
    pub kind: TokenKind,

    /// The slice of source file that produced this token
    pub slice: &'file str,

    /// The span in the source file
    pub span: FileSpan,
}

impl Token<'_> {
    pub fn is_whitespace(&self) -> bool {
        self.kind == TokenKind::Whitespace || self.kind == TokenKind::Comment
    }

    pub fn is_keyword(&self, slice: &str) -> bool {
        self.kind == TokenKind::Keyword && self.slice == slice
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{kind:?} @ {start}..{end} {slice:?}",
            kind = self.kind,
            start = self.span.start().to_usize(),
            end = self.span.end().to_usize() - 1,
            slice = self.slice,
        )
    }
}