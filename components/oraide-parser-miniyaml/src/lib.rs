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
