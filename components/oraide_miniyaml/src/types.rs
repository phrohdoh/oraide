use std::fmt;

use crate::{
    TokenCollectionExts as _,
};

use mltt_span::{
    Files,
    FileSpan,
};

use indextree::NodeId as ArenaNodeId;
pub type Arena<'file> = indextree::Arena<Node<'file>>;

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
        write!(f, "{kind:?} @ {start}..{end}",
            kind = self.kind,
            start = self.span.start().to_usize(),
            end = self.span.end().to_usize(),
        )?;

        match self.kind {
            TokenKind::Identifier | TokenKind::Comment => {
                write!(f, " {:?}", self.slice)?;
            },
            _ => {},
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<'file> {
    /// Token that makes up the whitespace before any other tokens
    /// (should always be a `Whitespace` kind)
    pub indentation_token: Option<Token<'file>>,

    /// Tokens that make up the *key* portion, if any
    pub key_tokens: Vec<Token<'file>>,

    /// The token (should always be a `:`) that separates
    /// the key from the comment / value / end-of-line
    // This must be `None` if `key_tokens` is empty, but can also be `None` for key-only nodes (`foo` is valid).
    pub key_terminator_token: Option<Token<'file>>,

    /// Tokens that make up the *value* portion, if any
    pub value_tokens: Vec<Token<'file>>,

    /// The comment token, if any
    pub comment_token: Option<Token<'file>>,
}

impl<'file> Node<'file> {
    pub(crate) fn empty() -> Self {
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
        self.indentation_token.as_ref().map_or(0, |token| token.slice.len())
    }

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
            whole_span_end = Some(span.end());
        }

        if let Some(span) = self.key_tokens.span() {
            source = Some(span.source());
            if whole_span_start.is_none() {
                whole_span_start = Some(span.start());
            }

            let span_end = span.end();
            match whole_span_end {
                Some(e) if e < span_end => whole_span_end = Some(span_end),
                None => whole_span_end = Some(span_end),
                _ => {}
            }

            if let Some(span) = self.key_terminator_token.as_ref().map(|token| token.span) {
                let span_end = span.end();
                match whole_span_end {
                    Some(e) if e < span_end => whole_span_end = Some(span_end),
                    None => whole_span_end = Some(span_end),
                    _ => {}
                }
            }
        }

        if let Some(span) = self.value_tokens.span() {
            let span_end = span.end();
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

            let span_end = span.end();
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

    pub fn key_slice(&self, files: &'file Files) -> Option<&'file str> {
        let span = match self.key_tokens.span() {
            Some(span) => span,
            _ => return None,
        };

        files.source(span)
    }

    pub fn slice(&self, files: &'file Files) -> Option<&'file str> {
        match self.span() {
            Some(span) => files.source(span),
            _ => None,
        }
    }
}

/// A struct that groups an `indextree::Arena` with all of its
/// `indextree::NodeId`s
pub struct Tree<'file> {
    /// All IDs for nodes that exist in `arena`, even the parent-less sentinel
    pub node_ids: Vec<ArenaNodeId>,

    /// The `indextree::Arena` that contains `Node`s
    pub arena: Arena<'file>,
}

impl<'file> Tree<'file> {
    /// Create a new instace of an `oraide_miniyaml::Tree`
    pub fn new(node_ids: Vec<ArenaNodeId>, arena: Arena<'file>) -> Self {
        Self {
            node_ids,
            arena,
        }
    }

    /// Get a tuple of `(arena_node_id, node)` for all top-level nodes, regardless of comment-only, whitespace-only, key-only, etc.
    pub fn top_level_nodes(&self) -> impl Iterator<Item=(ArenaNodeId, &Node<'file>)> {
        self.node_ids.iter().skip(1) // skip the sentinel
            .filter_map(move |nid| self.arena.get(*nid).map(|an| (*nid, &an.data)))
            .filter(|(_nid, shrd_node)| shrd_node.is_top_level())
    }
}

#[cfg(test)]
mod tests {
    use unindent::unindent;

    use mltt_span::{
        Files,
        FileSpan,
    };

    use crate::{
        Lexer,
        Parser,
    };

    #[test]
    fn span_is_correct() {
        // Arrange
        let src = unindent("
            A: # foo
                B:
                    C: auto
        ");

        let mut files = Files::new();
        let file_id = files.add("test", src);
        let file = &files[file_id];

        let lexer = Lexer::new(file);
        let tokens = lexer.collect::<Vec<_>>();

        let parser = Parser::new(tokens.into_iter());

        // Act
        let nodes = parser.collect::<Vec<_>>();

        // Assert
        assert_eq!(nodes.len(), 3);

        let mut nodes_iter = nodes.iter();

        let node_a = nodes_iter.next().unwrap();
        assert_eq!(node_a.span(), Some(FileSpan::new(file_id, 0, 8)));
        assert_eq!(node_a.slice(&files), Some("A: # foo"));

        let node_b = nodes_iter.next().unwrap();
        assert_eq!(node_b.span(), Some(FileSpan::new(file_id, 9, 15)));
        assert_eq!(node_b.slice(&files), Some("    B:"));

        let node_c = nodes_iter.next().unwrap();
        assert_eq!(node_c.span(), Some(FileSpan::new(file_id, 16, 31)));
        assert_eq!(node_c.slice(&files), Some("        C: auto"));
    }

    #[test]
    fn key_slice_is_correct() {
        // Arrange
        let src = unindent("
            Archer:mewo
                Barracks: woof
                    Cats #hoot
        ");

        let mut files = Files::new();
        let file_id = files.add("test", src);
        let file = &files[file_id];

        let lexer = Lexer::new(file);
        let tokens = lexer.collect::<Vec<_>>();

        let parser = Parser::new(tokens.into_iter());

        // Act
        let nodes = parser.collect::<Vec<_>>();

        // Assert
        assert_eq!(nodes.len(), 3);

        let mut nodes_iter = nodes.iter();

        let node_a = nodes_iter.next().unwrap();
        assert_eq!(node_a.key_slice(&files), Some("Archer"));

        let node_b = nodes_iter.next().unwrap();
        assert_eq!(node_b.key_slice(&files), Some("Barracks"));

        let node_c = nodes_iter.next().unwrap();
        assert_eq!(node_c.key_slice(&files), Some("Cats "));
    }
}