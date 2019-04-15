//! # `nodeizer`
//!
//! Transform a collection of `Token`s into a collection of `Node`s
//!
//! ---
//!
//! The entrypoint to this module is the `Nodeizer` struct.
//!

use itertools::MultiPeek;

use crate::{
    Token,
    TokenKind,
    Node,
};

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