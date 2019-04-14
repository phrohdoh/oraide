//! # `tokenizer`
//!
//! Demarcate text into a collection of `Token`s.
//!
//! See [Wikipedia][wikipedia] for more information.
//!
//! ---
//!
//! The entrypoint to this module is the `Tokenizer` struct.
//!
//! [wikipedia]: https://en.wikipedia.org/wiki/Lexical_analysis#Tokenization
//!

use std::{
    mem,
    str::{
        FromStr,
        Chars,
    },
};

use oraide_span::{
    FileId,
    FileSpan,
    ByteIndex,
    ByteCount,
};

use crate::{
    Token,
    TokenKind,
};

// free-standing functions that are composed to make up
// the complex MiniYaml syntax rules
fn is_symbol(ch: char) -> bool {
    match ch {
        '~' | '!' | '@' | ':' | '|' | '&' | '#' | '^' => true,
        _ => false,
    }
}

fn is_digit_or_numeric_symbol(ch: char) -> bool {
    match ch {
        '-' | '.' => true,
        _ if ch.is_digit(10) => true,
        _ => false,
    }
}

fn is_dec_digit_start(ch: char) -> bool {
    match ch {
        '-' => true, // support negative literals
        _ if ch.is_digit(10) => true,
        _ => false,
    }
}

#[inline(always)]
fn is_dec_digit_continue(ch: char) -> bool {
    is_digit_or_numeric_symbol(ch)
}

fn is_identifier_start(ch: char) -> bool {
    match ch {
        'a'..='z' | 'A'..='Z' | '_' => true,
        _ => false,
    }
}

fn is_identifier_continue(ch: char) -> bool {
    match ch {
        _ if is_dec_digit_continue(ch) => true, // `T01`, for example, is a valid identifier
        'a'..='z' | 'A'..='Z' | '_' | '-' | '.' => true,
        _ => false,
    }
}

/// Transform text into a collection of `Token`s for subsequent use
/// by a `Nodeizer` instance
///
/// # Lifetimes
/// `'text`: the underlying text that is being tokenized
///
/// # Example
/// ```rust
/// # use oraide_span::{FileId};
/// # use oraide_parser_miniyaml::{Token,Tokenizer};
/// let file_id = FileId(0);
/// let mut tokenizer = Tokenizer::new(file_id, "your source text");
/// let tokens: Vec<Token> = tokenizer.run();
/// assert_eq!(tokens.len(), 5);
/// ```
pub struct Tokenizer<'text> {
    /// The underlying text that is being tokenized
    text: &'text str,

    /// Used to manage `FileSpan`s
    file_id: FileId,

    /// An iterator of unicode characters to consume (initialized from `text`)
    chars: Chars<'text>,

    /// One character of lookahead (initialized from `chars`)
    peeked: Option<char>,

    /// The start position of the next token to be emitted
    token_start: ByteIndex,

    /// The end position (+ 1 byte) of the next token to be emitted
    token_end_exclusive: ByteIndex,
}

impl<'text> Tokenizer<'text> {
    /// Create a new `Tokenizer` from text and an associated `FileId`
    ///
    /// # Example
    ///
    /// ```rust
    /// # use oraide_span::{FileId};
    /// # use oraide_parser_miniyaml::{Token,Tokenizer};
    /// // Create the `Tokenizer`
    /// let mut tokenizer = Tokenizer::new(FileId(0), "your source text");
    ///
    /// // Tokenize the source text
    /// let tokens: Vec<Token> = tokenizer.run();
    ///
    /// // Quick sanity check
    /// assert_eq!(tokens.len(), 5);
    /// ```
    pub fn new(file_id: FileId, text: &'text str) -> Tokenizer<'text> {
        let mut chars = text.chars();
        let peeked = chars.next();

        Self {
            text,
            file_id,
            chars,
            peeked,
            token_start: ByteIndex(0),
            token_end_exclusive: ByteIndex(0),
        }
    }

    pub fn run(&mut self) -> Vec<Token> {
        self.by_ref().collect()
    }

    fn consume_token(&mut self) -> Option<TokenKind> {
        self.advance().map(|ch| match ch {
            // We put non-composite symbols here (instead of in `consume_symbol`)
            // so they don't get combined.
            '~' => TokenKind::Tilde,
            '!' => TokenKind::Bang,
            '@' => TokenKind::At,
            '^' => TokenKind::Caret,
            ':' => TokenKind::Colon,
            '-' if self.peek_satisfies(char::is_whitespace) => {
                // A `-` followed by whitespace is probably a pseudo
                // bullet-point string so treat it like a symbol.

                TokenKind::Symbol
            },
            '-' if self.peek_satisfies(is_identifier_start) => {
                // An identifier prefixed with a `-` (in MiniYaml this is
                // removing an inherited property) so just return the `-`
                // and let the next iteration get the identifier.

                // TODO: Consider a `Dash` variant.
                //       Need to think about the refactorings, etc., that
                //       an explicit Dash variant gives us (vs Symbol)

                TokenKind::Symbol
            },
            '\n' => TokenKind::EndOfLine,
            '\r' if self.peek_eq('\n') => {
                // Get the `\n` too
                self.advance();

                TokenKind::EndOfLine
            },
            '\r' => {
                // A `\r` not followed by `\n` is an invalid newline sequence
                // TODO: diagnostic
                TokenKind::Error
            },
            _ if is_symbol(ch) => self.consume_symbol(),
            _ if ch.is_whitespace() => {
                // Consume whitespace until end-of-line
                self.skip_while(|ch| ch != '\r' && ch != '\n' && ch.is_whitespace());
                TokenKind::Whitespace
            },

            // Identifiers are basically anything that fails to parse as an integer or float
            _ if is_dec_digit_start(ch) || is_identifier_start(ch) => self.consume_identifier_or_decimal_literal(),

            // Anything else, we can't realistically handle
            // (many human languages, etc.) so lump them into symbol
            _ => TokenKind::Symbol,
        })

    }

    /// Consume the current character and load the new one into the internal
    /// state, returning the just-consumed character
    fn advance(&mut self) -> Option<char> {
        let cur = mem::replace(&mut self.peeked, self.chars.next());
        self.token_end_exclusive += cur.map_or(ByteCount(0), ByteCount::from_char_len_utf8);
        cur
    }

    /// The next character, if any
    fn peek(&self) -> Option<char> {
        self.peeked
    }

    /// Query whether or not the next character, if any, is equal to `ch`
    fn peek_eq(&self, ch: char) -> bool {
        self.peek_satisfies(|c| c == ch)
    }

    /// Whether the next character, if any, satisifies `predicate`, returning `false` if there is no next character
    fn peek_satisfies(&self, predicate: impl FnMut(char) -> bool) -> bool {
        self.peek().map_or(false, predicate)
    }

    /// Consume a symbol
    fn consume_symbol(&mut self) -> TokenKind {
        self.skip_while(is_symbol);

        match self.token_slice() {
            "&&" => TokenKind::LogicalAnd,
            "||" => TokenKind::LogicalOr,
            slice if slice.starts_with("#") => {
                // Consume everything until we hit a newline sequence
                self.skip_while(|ch| ch != '\n' && ch != '\r');
                TokenKind::Comment
            },

            // This only happens if `skip_while` doesn't advance
            // which means we called this function when we shouldn't have,
            // i.e. when the peeked token wasn't actually a symbol
            // (as defined by `is_symbol`).
            slice if slice.is_empty() => {
                // TODO: diagnostic
                TokenKind::Error
            },
            _ => TokenKind::Symbol,
        }
    }

    /// Skip characters while the predicate matches the lookahead character.
    fn skip_while(&mut self, mut keep_going: impl FnMut(char) -> bool) {
        while self.peek().map_or(false, |ch| keep_going(ch)) {
            self.advance();
        }
    }

    /// Returns the string slice of the current token
    ///
    /// Panics if `self.token_start` or `self.token_end_exclusive` are out of bounds of `self.text`
    fn token_slice(&self) -> &'text str {
        let start = self.token_start.to_usize();
        let end_exclusive = self.token_end_exclusive.to_usize();
        &self.text[start..end_exclusive]
    }

    /// Consume either an identifier or a decimal literal
    fn consume_identifier_or_decimal_literal(&mut self) -> TokenKind {
        self.skip_while(|ch| is_identifier_continue(ch) || is_dec_digit_continue(ch));

        if self.token_slice().len() == 0 {
            // If this didn't advance then the next characters didn't satisfy
            // the above predicate which means we called this function
            // when we shouldn't have, this is an implementation bug.

            // TODO: diagnostic
            return TokenKind::Error;
        }

        let slice = self.token_slice();

        // keywords
        if slice.eq_ignore_ascii_case("true") { return TokenKind::True; }
        if slice.eq_ignore_ascii_case("false") { return TokenKind::False; }
        if slice.eq_ignore_ascii_case("yes") { return TokenKind::Yes; }
        if slice.eq_ignore_ascii_case("no") { return TokenKind::No; }

        // All `-`s is an identifier (really just "text", consider the value portion of a node)
        if itertools::all(slice.chars(), |ch| ch == '-') {
            return TokenKind::Identifier;
        }

        // If all the chars we have skipped so far are digits...
        if itertools::all(slice.chars(), is_digit_or_numeric_symbol) {
            // we're lexing a number (but it could be an int or a float, we don't know yet)

            if slice.chars().last().map_or(false, |ch| ch.is_digit(10)) {
                if slice.contains('.') {
                    return match f64::from_str(slice) {
                        Ok(_) => TokenKind::FloatLiteral,
                        Err(_) => {
                            log::debug!("Failed to parse text as signed 64-bit integer so assuming it is an identifier: {:?}", slice);
                            TokenKind::Identifier
                        },
                    };
                } else {
                    return match i64::from_str_radix(slice, 10) {
                        Ok(_) => TokenKind::IntLiteral,
                        Err(_) => {
                            log::debug!("Failed to parse text as signed 64-bit integer so assuming it is an identifier: {:?}", slice);
                            TokenKind::Identifier
                        },
                    };
                }
            }
        }

        TokenKind::Identifier
    }

    /// Emit a token and reset the start position, ready for the next token
    fn emit(&mut self, kind: TokenKind) -> Token {
        let span = self.token_span();
        self.token_start = self.token_end_exclusive;

        Token {
            kind,
            span,
        }
    }

    /// Returns a span in the underlying text
    fn span(&self, start: ByteIndex, end: ByteIndex) -> FileSpan {
        FileSpan::new(self.file_id, start, end)
    }

    /// Returns the span of the current token in the source file
    fn token_span(&self) -> FileSpan {
        self.span(self.token_start, self.token_end_exclusive)
    }
}

impl<'text> Iterator for Tokenizer<'text> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let opt_token = self.consume_token()
            .map(|kind| self.emit(kind));

        match &opt_token {
            Some(token) => log::trace!("emit {:?}", token),
            _ => log::trace!("end-of-input"),
        }

        opt_token
    }
}