// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! # `nodeizer`
//!
//! Transform a collection of `Token`s into a collection of `Node`s
//!
//! ---
//!
//! The entrypoint to this module is the `Nodeizer` struct.
//!

use itertools::MultiPeek;

use oraide_span::{
    FileSpan,
    ByteCount,
    ByteIndex,
};

use crate::{
    Token,
    TokenKind,
    TokenCollectionExts as _,
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

    pub fn key_span(&self) -> Option<FileSpan> {
        self.key_tokens.span()
    }

    /// Get the key-portion of a Node's text, if any exists
    pub fn key_text<'text>(&self, text: &'text str) -> Option<&'text str> {
        let span = match self.key_tokens.span() {
            None => return None,
            Some(s) => s,
        };

        let start = span.start().to_usize();
        if start >= text.len() {
            return None;
        }

        let end_exclusive = span.end_exclusive().to_usize();
        if end_exclusive >= text.len() {
            return None;
        }

        Some(&text[start..end_exclusive])
    }

    pub fn is_top_level(&self) -> bool {
        self.indentation_level() == 0
    }

    pub fn indentation_level(&self) -> usize {
        self.indentation_token.as_ref().map_or(0, |token| token.span.len().to_usize())
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

    /// Convert this [`Node`] into a collection of [`Token`]s
    ///
    /// [`Node`]: struct.Node.html
    /// [`Token`]: struct.Token.html
    pub fn into_tokens(mut self) -> Vec<Token> {
        let mut tokens = vec![];

        if let Some(t) = self.indentation_token {
            tokens.push(t);
        }

        tokens.append(&mut self.key_tokens);

        if let Some(t) = self.key_terminator_token {
            tokens.push(t);
        }

        tokens.append(&mut self.value_tokens);

        if let Some(t) = self.comment_token {
            tokens.push(t);
        }

        tokens
    }

    //// Compute the single [`Token`] that contains the given `span`, if any
    // pub fn token_containing_span(&self, span: FileSpan) -> Option<Token> {
    //     if let Some(cmp_span) = self.indentation_token.clone().map(|token| token.span) {
    //         if cmp_span.contains_span(span) {
    //             return self.indentation_token.clone();
    //         }
    //     }

    //     for shrd_key_token in self.key_tokens.iter() {
    //         if shrd_key_token.span.contains_span(span) {
    //             return Some(shrd_key_token.clone());
    //         }
    //     }

    //     if let Some(cmp_span) = self.key_terminator_token.clone().map(|token| token.span) {
    //         if cmp_span.contains_span(span) {
    //             return self.key_terminator_token.clone();
    //         }
    //     }

    //     for shrd_value_token in self.value_tokens.iter() {
    //         if shrd_value_token.span.contains_span(span) {
    //             return Some(shrd_value_token.clone());
    //         }
    //     }

    //     if let Some(cmp_span) = self.comment_token.clone().map(|token| token.span) {
    //         if cmp_span.contains_span(span) {
    //             return self.comment_token.clone();
    //         }
    //     }

    //     None
    // }
}

/// Transform a collection of [`Token`]s into a collection of [`Node`]s
///
/// # Type Parameters
/// `I`: An _iterable_ that yields [`Token`]s
///
/// # Example
/// ```rust
/// # use oraide_span::{FileId};
/// # use oraide_parser_miniyaml::{Token,Tokenizer,Nodeizer};
/// let mut tokenizer = Tokenizer::new(FileId(0), "your source text");
/// let tokens: Vec<Token> = tokenizer.run();
///
/// let nodeizer = Nodeizer::new(tokens.into_iter());
/// ```
pub struct Nodeizer<I: Iterator<Item = Token>> {
    tokens: MultiPeek<I>,
}

impl<I: Iterator<Item = Token>> Nodeizer<I> {
    /// Create a new `Nodeizer` from an `Iterator<Item = Token>`
    ///
    /// # Example
    /// ```rust
    /// # use oraide_span::{FileId};
    /// # use oraide_parser_miniyaml::{Token,Tokenizer,Nodeizer};
    /// let mut tokenizer = Tokenizer::new(FileId(0), "your source text");
    /// let tokens: Vec<Token> = tokenizer.run();
    ///
    /// let nodeizer = Nodeizer::new(tokens.into_iter());
    /// ```
    pub fn new(tokens: I) -> Nodeizer<I> {
        Self {
            tokens: itertools::multipeek(tokens),
        }
    }

    pub fn run(&mut self) -> Vec<Node> {
        self.by_ref().collect()
    }
}

impl<I: Iterator<Item = Token>> Iterator for Nodeizer<I> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let mut indentation_token: Option<Token> = None;
        let mut key_tokens = Vec::<Token>::new();
        let mut key_terminator_token: Option<Token> = None;
        let mut value_tokens = Vec::<Token>::new();
        let mut comment_token: Option<Token> = None;

        while let Some(token) = self.tokens.next() {
            match token.kind {
                TokenKind::EndOfLine => {
                    let node = Node {
                        indentation_token,
                        key_tokens,
                        key_terminator_token,
                        value_tokens,
                        comment_token
                    };

                    if node.indentation_token.is_some() || !node.key_tokens.is_empty() || !node.value_tokens.is_empty() || node.comment_token.is_some() {
                        log::trace!("emit {:#?}", node);
                    } else {
                        log::trace!("empty node");
                    }

                    return Some(node);
                }
                TokenKind::Comment => comment_token = Some(token),
                TokenKind::Whitespace => {
                    if key_tokens.is_empty() {
                        indentation_token = Some(token);
                    } else if key_terminator_token.is_some() {
                        value_tokens.push(token);
                    } else {
                        key_tokens.push(token);
                    }
                },
                TokenKind::Colon => {
                    if key_terminator_token.is_some() {
                        value_tokens.push(token);
                    } else {
                        // A colon being the first non-whitespace token is invalid
                        // because that is a key-less node (only empty or comment-only nodes can be key-less).
                        if !itertools::any(&key_tokens, |tok| tok.kind != TokenKind::Whitespace) {
                            // TODO: diagnostic
                        }

                        key_terminator_token = Some(token);
                    }
                },
                TokenKind::Bang => {
                    if key_terminator_token.is_some() {
                        value_tokens.push(token);
                    } else {
                        // TODO: diagnostic
                    }
                },
                TokenKind::At => {
                    if key_terminator_token.is_some() {
                        value_tokens.push(token);
                    } else {
                        match self.tokens.peek() {
                            Some(tok_peeked) if tok_peeked.kind != TokenKind::Identifier && !tok_peeked.is_numeric() => {
                                // TODO: diagnostic
                            },
                            _ => {},
                        }

                        self.tokens.reset_peek();
                        key_tokens.push(token);
                    }
                },
                TokenKind::Caret => {
                    if key_terminator_token.is_some() {
                        // A stand-alone caret in a value is potentially invalid
                        // (maybe the author forgot to finish typing the parent node name)
                        // but we can't know for sure.
                        //
                        // TODO: Think about how best to handle this, if at all.

                        match self.tokens.peek() {
                            // A ^ followed by a non-identifier, in value position, is *potentially* a typo
                            // TOOD: Once we have a "symbol table" of sorts we could remove this diag
                            //       as we'd do a lookup and, probably, not find an ident like `^!bar`.
                            Some(peeked_tok) if peeked_tok.kind != TokenKind::Identifier => {
                                // TODO: diagnostic
                            },
                            _ => {},
                        }

                        self.tokens.reset_peek();
                        value_tokens.push(token);
                    } else {
                        match self.tokens.peek() {
                            Some(peeked_tok) if peeked_tok.kind != TokenKind::Identifier => {
                                // TODO: diagnostic
                            },
                            None => { /* span end is eof */ },
                            _ => {},
                        }

                        self.tokens.reset_peek();

                        key_tokens.push(token);
                    }
                },
                _ => { // I don't yet know if handling the remaining
                       // variants like this is correct.
                    if key_terminator_token.is_some() {
                        value_tokens.push(token);
                    } else {
                        key_tokens.push(token);
                    }
                },
            }
        }

        None
    }
}

#[cfg(test)]
mod tests;