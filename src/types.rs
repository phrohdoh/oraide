use std::fmt;

use itertools::Itertools;
use mltt_span::{
    FileSpan,
};

/// A tag that makes it easier to store what type of token this is
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Error,

    // ignorables
    Whitespace,
    Comment,

    // keywords
    True,
    Yes,
    False,
    No,
    // TODO: Consider adding `Inherits`
 //Inherits,

    // literals / free-form words
    Identifier,
    IntLiteral,
    FloatLiteral,

    // symbols
    Symbol,
    Tilde,
    Bang,
    At,
    Caret,
    Colon,
    LogicalOr,
    LogicalAnd,

    Eol,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            TokenKind::Error => "<*error*>",
            TokenKind::Whitespace => "<whitespace>",
            TokenKind::Comment => "<comment>",
            TokenKind::True => "true",
            TokenKind::Yes => "yes",
            TokenKind::False => "false",
            TokenKind::No => "no",
            TokenKind::Identifier => "<identifier>",
            TokenKind::IntLiteral => "<integer literal>",
            TokenKind::FloatLiteral => "<float literal>",
            TokenKind::Symbol => "<symbol>",
            TokenKind::Tilde => "~",
            TokenKind::Bang => "!",
            TokenKind::At => "@",
            TokenKind::Caret => "^",
            TokenKind::Colon => ":",
            TokenKind::LogicalOr => "||",
            TokenKind::LogicalAnd => "&&",
            TokenKind::Eol => "<end-of-line>",
        })
    }
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
        self.kind == TokenKind::Whitespace
            || self.kind == TokenKind::Eol
            || self.kind == TokenKind::Comment
    }

    pub fn is_symbol(&self) -> bool {
        match self.kind {
              TokenKind::Symbol
            | TokenKind::Tilde
            | TokenKind::Bang
            | TokenKind::At
            | TokenKind::Caret
            | TokenKind::Colon
            | TokenKind::LogicalOr
            | TokenKind::LogicalAnd => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        self.kind == TokenKind::IntLiteral || self.kind == TokenKind::FloatLiteral
    }

    pub fn is_keyword(&self, slice: &str) -> bool {
        match self.kind {
            TokenKind::True
          | TokenKind::Yes
          | TokenKind::False
          | TokenKind::No => self.slice == slice,
          _ => false
        }
    }
}

impl fmt::Debug for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{kind:?} @ {start}..{end} {slice:?}",
            kind = self.kind,
            start = self.span.start().to_usize(),
            end = self.span.end().to_usize(),
            slice = self.slice,
        )
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Node<'file> {
    /// Tokens that make up the whitespace before any other tokens
    pub indentation_tokens: Vec<Token<'file>>,

    /// Tokens that make up the *key* portion, if any
    pub key_tokens: Vec<Token<'file>>,

    /// Tokens that make up the *value* portion, if any
    pub value_tokens: Vec<Token<'file>>,

    /// The comment token, if any
    pub comment_token: Option<Token<'file>>,
}

impl fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.indentation_tokens.is_empty() {
            write!(f, "indentation=[{}]", self.indentation_tokens
                .iter()
                .map(|tok| tok.kind)
                .join(" "))?;
        }

        if !self.key_tokens.is_empty() {
            if !self.indentation_tokens.is_empty() {
                write!(f, " ")?;
            }

            write!(f, "key=[{}]", self.key_tokens
                .iter()
                .map(|tok| tok.kind)
                .join(" "))?;
        }

        if !self.value_tokens.is_empty() {
            if !self.key_tokens.is_empty() {
                write!(f, " ")?;
            }

            write!(f, "value=[{}]", self.value_tokens
                .iter()
                .map(|tok| tok.kind)
                .join(" "))?;
        }

        if self.comment_token.is_some() {
            if !self.value_tokens.is_empty() {
                write!(f, " ")?;
            }

            write!(f, "<comment>")?;
        }

        Ok(())
    }
}