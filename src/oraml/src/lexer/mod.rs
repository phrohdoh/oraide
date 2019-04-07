use std::{
    str::{
        FromStr,
        Chars,
    },
};

use language_reporting::{
    Diagnostic,
    Label,
};

use mltt_span::{
    ByteIndex,
    ByteSize,
    File,
    FileSpan,
};

use crate::types::{
    Token,
    TokenKind,
};

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

/// An iterator over a source string yielding `Token`s for subsequent use by
/// a `Parser` instance.
pub struct Lexer<'file> {
    /// The file being lexed
    file: &'file File,

    /// An iterator of unicode characters to consume
    chars: Chars<'file>,

    /// One character of lookahead
    peeked: Option<char>,

    /// Start position of the next token to be emitted
    token_start: ByteIndex,

    /// End position of the next token to be emitted
    // I *think* this is actually "end + 1", see https://gitter.im/pikelet-lang/Lobby?at=5c65912a28c89123cbcb0614
    token_end: ByteIndex,

    /// Diagnostics accumulated during lexing
    diagnostics: Vec<Diagnostic<FileSpan>>,
}

impl<'file> Lexer<'file> {
    /// Create a new `Lexer` from a source file
    pub fn new(file: &'file File) -> Lexer<'file> {
        let mut chars = file.contents().chars();
        let peeked = chars.next();

        Self {
            file,
            chars,
            peeked,
            token_start: ByteIndex::from(0),
            token_end: ByteIndex::from(0),
            diagnostics: Vec::new(),
        }
    }

    /// Record a diagnostic
    fn add_diagnostic(&mut self, diagnostic: Diagnostic<FileSpan>) {
        log::debug!("diagnostic added ({}) @ {}..{}: {:?}", diagnostic.severity.to_str(), self.token_span().start().to_usize(), self.token_span().end().to_usize(), diagnostic.message);
        self.diagnostics.push(diagnostic);
    }

    /// Take the diagnostics from the lexer, leaving an empty collection
    pub fn take_diagnostics(&mut self) -> Vec<Diagnostic<FileSpan>> {
        std::mem::replace(&mut self.diagnostics, Vec::new())
    }

    /// The next character, if any
    fn peek(&self) -> Option<char> {
        self.peeked
    }

    /// Query whether or not the next character, if any, is equal to `ch`
    fn peek_eq(&self, ch: char) -> bool {
        self.peek_satisfies(|c| c == ch)
    }

    /// Query whether or not the next character, if any, satisifies `predicate`, returning `false` if there is no next character
    fn peek_satisfies(&self, predicate: impl FnMut(char) -> bool) -> bool {
        self.peek().map_or(false, predicate)
    }

    /// Consume the current character and load the new one into the internal state, returning the just-consumed character
    fn advance(&mut self) -> Option<char> {
        let cur = std::mem::replace(&mut self.peeked, self.chars.next());
        // TODO: This causes single-char tokens to have a span of 2 bytes
        // though this may be intentional (see the non-doc comment on self.token_end).
        self.token_end += cur.map_or(ByteSize::from(0), ByteSize::from_char_len_utf8);
        cur
    }

    /// Returns a span in the source file
    fn span(&self, start: ByteIndex, end: ByteIndex) -> FileSpan {
        FileSpan::new(self.file.id(), start, end)
    }

    /// Returns the span of the current token in the source file
    fn token_span(&self) -> FileSpan {
        self.span(self.token_start, self.token_end)
    }

    /// Returns the string slice of the current token
    ///
    /// Panics if `self.token_start` or `self.token_end` are out of bounds of `self.file.contents()`
    fn token_slice(&self) -> &'file str {
        &self.file.contents()[self.token_start.to_usize()..self.token_end.to_usize()]
    }

    /// Emit a token and reset the start position, ready for the next token
    fn emit(&mut self, kind: TokenKind) -> Token<'file> {
        let slice = self.token_slice();
        let span = self.token_span();
        self.token_start = self.token_end;

        Token {
            kind,
            slice,
            span,
        }
    }

    /// Consume a token, returning its tag or `None` if end-of-file has been reached
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
            '\n' => TokenKind::Eol,
            '\r' if self.peek_eq('\n') => {
                // Get the `\n` too
                self.advance();

                TokenKind::Eol
            },
            '\r' => {
                // A `\r` not followed by `\n` is an invalid newline sequence
                self.add_diagnostic(
                    // warning, not error, because we can continue lexing
                    Diagnostic::new_warning("invalid newline sequence")
                        .with_code("W0004")
                        .with_label(Label::new_primary(self.token_span()))
                );

                TokenKind::Error
            },
            _ if is_symbol(ch) => self.consume_symbol(),
            _ if ch.is_whitespace() => self.consume_whitespace_until_eol(),

            // Identifiers are basically anything that fails to parse as an integer or float
            _ if is_dec_digit_start(ch) || is_identifier_start(ch) => self.consume_identifier_or_decimal_literal(),

            // Anything else, we can't realistically handle
            // (many human languages, etc.) so lump them into symbol
            _ => TokenKind::Symbol,
        })
    }

    /// Consume a symbol
    fn consume_symbol(&mut self) -> TokenKind {
        self.skip_while(is_symbol);

        match self.token_slice() {
            "&&" => TokenKind::LogicalAnd,
            "||" => TokenKind::LogicalOr,
            slice if slice.starts_with("#") => self.consume_comment(),

            // This only happens if `skip_while` doesn't advance
            // which means we called this function when we shouldn't have,
            // i.e. when the peeked token wasn't actually a symbol
            // (as defined by `is_symbol`).
            slice if slice.is_empty() => {
                self.add_diagnostic(
                    Diagnostic::new_bug(format!(
                        "{}::{} invoked with invalid Lexer state, expected next charater to be a symbol",
                        stringify!(Lexer),
                        stringify!(consume_symbol)
                    )).with_code("L:B0001")
                );

                TokenKind::Error
            },
            _ => TokenKind::Symbol,
        }
    }

    /// Consume everything until we hit a newline sequence
    fn consume_comment(&mut self) -> TokenKind {
        self.skip_while(|ch| ch != '\n' && ch != '\r');

        TokenKind::Comment
    }

    /// Consume either an identifier or a decimal literal
    fn consume_identifier_or_decimal_literal(&mut self) -> TokenKind {
        self.skip_while(|ch| is_identifier_continue(ch) || is_dec_digit_continue(ch));

        if self.token_span().len() == 0.into() {
            // If this didn't advance then the next characters didn't satisfy
            // the above predicate which means we called this function
            // when we shouldn't have, this is an implementation bug.

            self.add_diagnostic(
                Diagnostic::new_bug(format!(
                    "{}::{} invoked with invalid Lexer state, expected next character(s) to satisfy `{}` or `{}`",
                    stringify!(Lexer),
                    stringify!(consume_identifier),
                    stringify!(is_identifier_continue),
                    stringify!(is_dec_digit_continue),
                )).with_code("L:B0004")
            );

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

    /// Consume whitespace
    fn consume_whitespace_until_eol(&mut self) -> TokenKind {
        self.skip_while(|ch| ch != '\r' && ch != '\n' && ch.is_whitespace());
        TokenKind::Whitespace
    }

    /// Skip characters while the predicate matches the lookahead character.
    fn skip_while(&mut self, mut keep_going: impl FnMut(char) -> bool) {
        while self.peek().map_or(false, |ch| keep_going(ch)) {
            self.advance();
        }
    }
}

/// This is where the magic happens.
///
/// `Lexer`-using code will call `lexer.collect()` to actually run the lexer
/// and collect the resultant token stream.
impl<'file> Iterator for Lexer<'file> {
    type Item = Token<'file>;

    fn next(&mut self) -> Option<Self::Item> {
        let opt_token = self.consume_token()
            .map(|tag| self.emit(tag));

        match &opt_token {
            Some(token) => log::debug!("emit {:?}", token),
            _ => log::debug!("eof"),
        }

        opt_token
    }
}

#[cfg(test)]
mod tests;