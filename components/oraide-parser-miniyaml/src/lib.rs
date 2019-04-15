//! # `oraide-parser-miniyaml`
//!
//! Convert textual MiniYaml documents into MiniYaml trees
//!

use indextree::{
    NodeId as ArenaNodeId,
};

pub type Arena = indextree::Arena<Node>;

use oraide_span::{
    FileSpan,
};

mod parser;

pub use parser::{
    Token,
    TokenKind,
    Tokenizer,
    TokenCollectionExts,
    Nodeizer,
    IndentLevelDelta,
    Treeizer,
};

/// A [`Node`] is, in the context of textual MiniYaml, a structured collection
/// of [`Token`]s in a single line
///
/// [`Token`]: struct.Token.html
/// [`Node`]: struct.Node.html
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node {
    /// Token that makes up the whitespace before any other tokens,
    /// if any (should always be a [`Whitespace`] kind)
    ///
    /// [`Whitespace`]: enum.TokenKind.html#variant.Whitespace
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

/// A [`Tree`] groups an [`indextree::Arena`] with all of its [`indextree::NodeId`]s
///
/// [`Tree`]: struct.Tree.html
/// [`indextree::Arena`]: ../indextree/struct.Arena.html
#[derive(Debug, Clone)]
pub struct Tree {
    /// All IDs for nodes that exist in `arena` with the first item always
    /// being the sentinel for parent-less nodes
    pub node_ids: Vec<ArenaNodeId>,

    /// The `indextree::Arena` that contains `Node`s
    pub arena: Arena,
}

impl Tree {
    pub fn from(node_ids: Vec<ArenaNodeId>, arena: Arena) -> Self {
        Self {
            node_ids,
            arena,
        }
    }
}