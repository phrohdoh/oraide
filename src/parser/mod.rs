use itertools::MultiPeek;

use language_reporting::{
    Diagnostic,
    Label,
};

use mltt_span::{
    FileId,
    FileSpan,
};

use crate::{
    types::{
        Token,
        TokenKind,
        Node,
    },
};

/// Transform a stream of tokens into a stream of line-based nodes
pub struct Parser<Tokens: Iterator> {
    /// A handle to the file we're parsing
    file_id: FileId,

    /// The underlying stream of tokens
    tokens: MultiPeek<Tokens>,

    /// Diagnostics accumulated during parsing
    diagnostics: Vec<Diagnostic<FileSpan>>,
}

impl<'file, Tokens> Parser<Tokens>
    where Tokens: Iterator<Item = Token<'file>> + 'file,
{
    /// Create a new parser from an iterator of `Token`s
    pub fn new(file_id: FileId, tokens: Tokens) -> Parser<Tokens> {
        Self {
            file_id,
            tokens: itertools::multipeek(tokens),
            diagnostics: vec![],
        }
    }

    /// Record a diagnostic
    fn add_diagnostic(&mut self, diagnostic: Diagnostic<FileSpan>) {
        self.diagnostics.push(diagnostic);
    }

    /// Take the diagnostics from the parser, leaving an empty collection
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic<FileSpan>> {
        std::mem::replace(&mut self.diagnostics, Vec::new())
    }
}

impl<'file, Tokens> Iterator for Parser<Tokens>
    where Tokens: Iterator<Item = Token<'file>> + 'file,
{
    type Item = Node<'file>;

    // An iteration finishes when a node is fully-formed
    fn next(&mut self) -> Option<Self::Item> {
        let mut indentation_tokens = Vec::<Token<'_>>::new();
        let mut key_tokens = Vec::<Token<'_>>::new();
        let mut key_terminator_token: Option<Token<'_>> = None;
        let mut value_tokens = Vec::<Token<'_>>::new();
        let mut comment_token: Option<Token<'_>> = None;

        while let Some(token) = self.tokens.next() {
            match token.kind {
                TokenKind::Eol => {
                    let node = Node {
                        indentation_tokens,
                        key_tokens,
                        key_terminator_token,
                        value_tokens,
                        comment_token
                    };

                    if !node.indentation_tokens.is_empty() || !node.key_tokens.is_empty() || !node.value_tokens.is_empty() || node.comment_token.is_some() {
                        log::debug!("emit {:?}", node);
                    } else {
                        log::debug!("empty node");
                    }

                    return Some(node);
                }
                TokenKind::Comment => comment_token = Some(token),
                TokenKind::Whitespace => {
                    if key_tokens.is_empty() {
                        indentation_tokens.push(token);
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
                            let span_colon = token.span.clone();

                            self.add_diagnostic(Diagnostic::new_error("No key found for this node")
                                .with_code("P:E0002")
                                .with_label(Label::new_primary(span_colon))
                            );

                            // Try to end the span *just before* the newline
                            // so the printed message doesn't include a newline
                            // which makes it look weird.
                            let opt_tok_eol = loop {
                                // Peek until eol which will give us the node end position
                                match self.tokens.peek() {
                                    Some(tok) if tok.kind == TokenKind::Eol => {
                                        break Some(tok);
                                    },
                                    None => {
                                        // eof
                                        break None;
                                    },
                                    _ => {},
                                }
                            };

                            // We got to the end of the file
                            if opt_tok_eol.is_none() {
                                // TODO: Consider adding an explicit `Eof` variant to `TokenKind`
                                unimplemented!("node_span_end.is_none() = true, TODO: get eof location");
                            }

                            let mut diag = Diagnostic::new_note("Nodes must be entirely empty, have a key, or have a comment, they can not be value-only");

                            if let Some(tok_eol) = opt_tok_eol {
                                let span = FileSpan::new(self.file_id, span_colon.start(), tok_eol.span.start());
                                diag = diag.with_label(Label::new_secondary(span));
                            }

                            // This must be done after the last usage of `opt_tok_eol` or
                            // we end up borrowing `self.tokens` as mutable multiple times.
                            self.tokens.reset_peek();

                            self.add_diagnostic(diag);
                        }

                        key_terminator_token = Some(token);
                    }
                },
                TokenKind::Bang => {
                    if key_terminator_token.is_some() {
                        value_tokens.push(token);
                    } else {
                        self.add_diagnostic(
                            Diagnostic::new_error("`!` can not be used in a node's key")
                                .with_code("P:E0004")
                                .with_label(Label::new_primary(token.span.clone()))
                        );

                        self.add_diagnostic(
                            Diagnostic::new_help("remove this `!` symbol")
                                .with_label(Label::new_secondary(token.span.clone()))
                        );

                        self.add_diagnostic(
                            Diagnostic::new_note("`!` can be used in strings or in conditionals to negate a boolean value")
                        );
                    }
                },
                TokenKind::At => {
                    if key_terminator_token.is_some() {
                        value_tokens.push(token);
                    } else {
                        match self.tokens.peek() {
                            Some(tok_peeked) if tok_peeked.kind != TokenKind::Identifier && !tok_peeked.is_number() => {
                                let diag_span = tok_peeked.span.clone();
                                self.add_diagnostic(Diagnostic::new_error("expected an identifier or number after `@`")
                                    .with_code("P:E0003")
                                    .with_label(Label::new_primary(diag_span))
                                );

                                self.add_diagnostic(
                                    Diagnostic::new_note("valid examples: `MyProperty@hello`, `HelloWorld@3`")
                                );
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
                            // TODO: Once we have a "symbol table" of sorts we could remove this diag
                            //       as we'd do a lookup and, probably, not find an ident like `^!bar`.
                            Some(peeked_tok) if peeked_tok.kind != TokenKind::Identifier => {
                                let peeked_span = peeked_tok.span;

                                self.add_diagnostic(
                                    Diagnostic::new_warning("A caret followed by a non-identifier is potentially a typo")
                                        .with_code("P:W0001")
                                        .with_label(Label::new_primary(token.span))
                                );

                                self.add_diagnostic(
                                    Diagnostic::new_help("consider removing this")
                                        .with_label(Label::new_secondary(peeked_span))
                                );
                            },
                            _ => {},
                        }

                        self.tokens.reset_peek();
                        value_tokens.push(token);
                    } else {
                        match self.tokens.peek() {
                            Some(peeked_tok) if peeked_tok.kind != TokenKind::Identifier => {
                                let mut diags_to_add = vec![];

                                let (peeked_kind_str, peeked_span) = {
                                    let peeked_kind_str = match peeked_tok.kind {
                                        TokenKind::Whitespace => "whitespace",
                                        TokenKind::Eol => "newline",
                                        _ if peeked_tok.is_symbol() => "symbol",
                                        _ if peeked_tok.is_keyword(peeked_tok.slice) => {
                                            // Can't use `add_diagnostic` here because that would be a double-mut borrow
                                            // of `self` due to `self.tokens.peek` taking `&mut self`.
                                            diags_to_add.push(Diagnostic::<FileSpan>::new_note(
                                                "keywords have special meaning and can not be used as keys"
                                            ));

                                            "keyword"
                                        },
                                        _ => "text",
                                    };

                                    (peeked_kind_str, peeked_tok.span.clone())
                                };

                                self.add_diagnostic(Diagnostic::new_error("expected an identifier after `^`")
                                    .with_code("P:E0001")
                                    .with_label(Label::new_primary(token.span.clone()))
                                );

                                self.add_diagnostic(Diagnostic::new_help(format!(
                                    "remove this {}",
                                    peeked_kind_str
                                )).with_label(Label::new_secondary(peeked_span)));

                                for diag in diags_to_add {
                                    self.add_diagnostic(diag);
                                }

                            },
                            None => { /* span end is eof */ },
                            _ => {},
                        }

                        self.tokens.reset_peek();

                        key_tokens.push(token);
                    }
                },
                TokenKind::Identifier => {
                    if key_terminator_token.is_some() {
                        value_tokens.push(token);
                    } else {
                        key_tokens.push(token);
                    }
                },
                TokenKind::True => {},
                _ => unimplemented!("{:?}", token),
            }
        }

        None
    }
}

#[cfg(test)]
mod tests;