use std::iter::Peekable;

use language_reporting::{
    Diagnostic,
    Label,
};

use mltt_span::{
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
    /// The underlying stream of tokens
    tokens: Peekable<Tokens>,

    /// Diagnostics accumulated during parsing
    diagnostics: Vec<Diagnostic<FileSpan>>,
}

impl<'file, Tokens> Parser<Tokens>
    where Tokens: Iterator<Item = Token<'file>> + 'file,
{
    /// Create a new parser from an iterator of `Token`s
    pub fn new(tokens: Tokens) -> Parser<Tokens> {
        Self {
            tokens: tokens.peekable(),
            diagnostics: vec![],
        }
    }

    /// The next token, if any
    fn peek(&mut self) -> Option<&Tokens::Item> {
        self.tokens.peek()
    }

    /// Query whether the next token's `kind` field is *equal to* `kind`,
    /// returning `false` if there is no next token
    #[allow(dead_code)]
    fn peek_kind_eq(&mut self, kind: TokenKind) -> bool {
        self.peek().map_or(false, |tok| tok.kind == kind)
    }

    /// Query whether the next token's `kind` field is *not equal to* `kind`,
    /// returning `false` if there is no next token
    fn peek_kind_ne(&mut self, kind: TokenKind) -> bool {
        self.peek().map_or(false, |tok| tok.kind != kind)
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
        let mut have_parsed_colon = false;
        let mut key_tokens = Vec::<Token<'_>>::new();
        let mut value_tokens = Vec::<Token<'_>>::new();
        let mut comment_token: Option<Token<'_>> = None;

        while let Some(token) = self.tokens.next() {
            match token.kind {
                TokenKind::Eol => {
                    let node = Node {
                        key_tokens,
                        value_tokens,
                        comment_token
                    };

                    log::debug!("emit {:?}", node);

                    return Some(node);
                }
                TokenKind::Comment => comment_token = Some(token),
                TokenKind::Whitespace => {
                    if have_parsed_colon {
                        value_tokens.push(token);
                    } else {
                        key_tokens.push(token);
                    }
                },
                TokenKind::Colon => {
                    if have_parsed_colon {
                        value_tokens.push(token);
                    } else {
                        // A colon being the first non-whitespace token is invalid
                        // because that is a key-less node (only empty or comment-only nodes can be key-less).
                        if !itertools::any(&key_tokens, |tok| tok.kind != TokenKind::Whitespace) {
                            let span = token.span.clone();

                            self.add_diagnostic(Diagnostic::new_error("No key found for this node")
                                .with_code("P:E0002")
                                .with_label(Label::new_primary(span))
                            );

                            self.add_diagnostic(Diagnostic::new_note(
                                "Nodes must be entirely empty, have a key, or have a comment, they can not be value-only"
                                // TODO: Attach the span of the entire node.
                                // Could probably use `itertools::multipeek`'s `peek` in a while loop
                                // to find the Eol token which gives us the entire span.
                            ));
                        }

                        have_parsed_colon = true;
                    }
                },
                TokenKind::Bang => {
                    if have_parsed_colon {
                        value_tokens.push(token);
                    } else {
                        key_tokens.push(token);
                    }
                },
                TokenKind::At => {
                    if have_parsed_colon {
                        value_tokens.push(token);
                    } else {
                        key_tokens.push(token);
                    }
                },
                TokenKind::Caret => {
                    if have_parsed_colon {
                        // A stand-alone caret in a value is potentially invalid
                        // (maybe the author forgot to finish typing the parent node name)
                        // but we can't know for sure.
                        //
                        // TODO: Think about how best to handle this, if at all.
                        value_tokens.push(token);
                    } else {
                        if self.peek_kind_ne(TokenKind::Identifier) {
                            let mut diags_to_add = vec![];

                            let (peeked_kind_str, peeked_span) = {
                                // This `unwrap` is fine, `peek_kind_ne` would
                                // return `false` if there is no token to peek
                                // so we wouldn't even be executing these lines.
                                let p = self.peek().unwrap();

                                let peeked_kind_str = match p.kind {
                                    TokenKind::Whitespace => "whitespace",
                                    TokenKind::Eol => "newline",
                                    _ if p.is_symbol() => "symbol",
                                    _ if p.is_keyword(p.slice) => {
                                        // Can't use `add_diagnostic` here because that would be a double-mut borrow
                                        // of `self` due to `peek` taking `&mut self`.
                                        diags_to_add.push(Diagnostic::<FileSpan>::new_note(
                                            "keywords have special meaning and can not be used as keys"
                                        ));

                                        "keyword"
                                    },
                                    _ => "text",
                                };

                                (peeked_kind_str, p.span.clone())
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
                        }

                        key_tokens.push(token);
                    }
                },
                TokenKind::Identifier => {
                    if have_parsed_colon {
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