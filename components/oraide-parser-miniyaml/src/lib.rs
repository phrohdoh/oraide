//! # `oraide-parser-miniyaml`
//!
//! Convert textual MiniYaml documents into MiniYaml trees
//!

use oraide_span::{
    FileSpan,
};

mod parser;

pub use parser::{
    Tokenizer,
    Nodeizer,
};

/// Used to indicate which type of `Token` this is
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenKind {
    // something went wrong and we aren't certain which
    // token kind this should be recorded as
    Error,

    Whitespace,
    Comment,

    // keywords
    True,
    Yes,
    False,
    No,

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

    EndOfLine,
}

/// A `Token` is the smallest unit of meaning in text parsing.
///
/// # Example
///
/// ```rust
/// # use oraide_span::{FileId,FileSpan};
/// # use oraide_parser_miniyaml::{Token,TokenKind,Tokenizer};
/// // Required to create a `Tokenizer`
/// let file_id = FileId(0);
///
/// // Create the `Tokenizer`
/// let mut tokenizer = Tokenizer::new(file_id, "your source text");
///
/// // Tokenize the source text
/// let tokens: Vec<Token> = tokenizer.run();
///
/// // Quick sanity check
/// assert_eq!(tokens.len(), 5);
///
/// // Verify the contents of the 1st token
/// let first_token = tokens.first().unwrap();
/// assert_eq!(first_token.kind, TokenKind::Identifier);
/// assert_eq!(first_token.span, FileSpan::new(file_id, 0, 4));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    /// The kind of token located at `span`
    pub kind: TokenKind,

    /// Where, in a file, this token is located
    pub span: FileSpan,
}

impl Token {
    fn is_whitespace(&self) -> bool {
        self.kind == TokenKind::Whitespace
            || self.kind == TokenKind::EndOfLine
            || self.kind == TokenKind::Comment
    }

    fn is_symbol(&self) -> bool {
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

    fn is_numeric(&self) -> bool {
        self.kind == TokenKind::IntLiteral || self.kind == TokenKind::FloatLiteral
    }

    fn is_keyword(&self) -> bool {
        match self.kind {
            TokenKind::True
          | TokenKind::Yes
          | TokenKind::False
          | TokenKind::No => true,
          _ => false
        }
    }
}

pub trait TokenCollectionExts {
    /// Get a slice of `Token`s that starts *after* leading `TokenKind::Whitespace`s
    fn skip_leading_whitespace(&self) -> &[Token];

    /// Get a span covering the entire collection of `Token`s
    ///
    /// Typically this is used to get the span of a single node (which, in practice, is an entire line)
    fn span(&self) -> Option<FileSpan>;
}

impl TokenCollectionExts for [Token] {
    fn skip_leading_whitespace(&self) -> &[Token] {
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
        let end = self.last().unwrap().span.end_exclusive();

        Some(FileSpan::new(first.span.source(), start, end))
    }
}

/// A `Node` is, in the context of textual MiniYaml, a structured collection
/// of `Token`s in a single line
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node {
    /// Token that makes up the whitespace before any other tokens,
    /// if any (should always be a `Whitespace` kind)
    pub indentation_token: Option<Token>,

    /// Tokens that make up the *key* portion, if any
    pub key_tokens: Vec<Token>,

    /// The token (should always be a `:`) that separates
    /// the key from the comment / value / end-of-line
    // This must be `None` if `key_tokens` is empty, but can also be `None` for key-only nodes (`foo` is valid).
    pub key_terminator_token: Option<Token>,

    /// Tokens that make up the *value* portion, if any
    pub value_tokens: Vec<Token>,

    /// The comment token, if any
    pub comment_token: Option<Token>,
}

impl Node {
    pub(crate) fn new_empty() -> Self {
        Self {
            indentation_token: None,
            key_tokens: vec![],
            key_terminator_token: None,
            value_tokens: vec![],
            comment_token: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.indentation_token.is_none()
        && self.key_tokens.is_empty()
        && self.key_terminator_token.is_none()
        && self.value_tokens.is_empty()
        && self.comment_token.is_none()
    }

    pub fn is_whitespace_only(&self) -> bool {
        self.indentation_token.is_some()
        && self.key_tokens.is_empty()
        && self.key_terminator_token.is_none()
        && self.value_tokens.is_empty()
        && self.comment_token.is_none()
    }

    pub fn is_comment_only(&self) -> bool {
        self.comment_token.is_some()
        && self.key_tokens.is_empty()
        && self.key_terminator_token.is_none()
        && self.value_tokens.is_empty()
    }

    pub fn has_key(&self) -> bool {
        !self.key_tokens.is_empty()
    }

    pub fn is_top_level(&self) -> bool {
        self.indentation_level() == 0
    }

    pub fn indentation_level(&self) -> usize {
        self.indentation_token.as_ref().map_or(0, |token| token.span.len().to_usize())
    }

    // TODO: Change this to return `Option<Span<_>>`
    pub fn span(&self) -> Option<FileSpan> {
        if self.is_empty() {
            return None;
        }

        let mut source = None;
        let mut whole_span_start = None;
        let mut whole_span_end = None;

        if let Some(span) = self.indentation_token.as_ref().map(|token| token.span) {
            source = Some(span.source());
            whole_span_start = Some(span.start());
            whole_span_end = Some(span.end_exclusive());
        }

        if let Some(span) = self.key_tokens.span() {
            source = Some(span.source());
            if whole_span_start.is_none() {
                whole_span_start = Some(span.start());
            }

            let span_end = span.end_exclusive();
            match whole_span_end {
                Some(e) if e < span_end => whole_span_end = Some(span_end),
                None => whole_span_end = Some(span_end),
                _ => {}
            }

            if let Some(span) = self.key_terminator_token.as_ref().map(|token| token.span) {
                let span_end = span.end_exclusive();
                match whole_span_end {
                    Some(e) if e < span_end => whole_span_end = Some(span_end),
                    None => whole_span_end = Some(span_end),
                    _ => {}
                }
            }
        }

        if let Some(span) = self.value_tokens.span() {
            let span_end = span.end_exclusive();
            match whole_span_end {
                Some(e) if e < span_end => whole_span_end = Some(span_end),
                None => whole_span_end = Some(span_end),
                _ => {}
            }
        }

        if let Some(span) = self.comment_token.as_ref().map(|token| token.span) {
            source = Some(span.source());
            if whole_span_start.is_none() {
                whole_span_start = Some(span.start());
            }

            let span_end = span.end_exclusive();
            match whole_span_end {
                Some(e) if e < span_end => whole_span_end = Some(span_end),
                None => whole_span_end = Some(span_end),
                _ => {}
            }
        }

        Some(match (source, whole_span_start, whole_span_end) {
            (Some(source), Some(start), Some(end)) => FileSpan::new(source, start, end),
            _ => return None,
        })
    }
}