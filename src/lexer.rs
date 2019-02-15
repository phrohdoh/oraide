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

fn is_dec_digit_start(ch: char) -> bool {
    match ch {
        '-' => true, // support negative literals
        _ if ch.is_digit(10) => true,
        _ => false,
    }
}

fn is_dec_digit_continue(ch: char) -> bool {
    ch.is_digit(10)
}

fn is_identifier_start(ch: char) -> bool {
    match ch {
        'a'..='z' | 'A'..='Z' | '_' => true,
        _ => false,
    }
}

fn is_identifier_continue(ch: char) -> bool {
    match ch {
        'a'..='z' | 'A'..='Z' | '_' | '-' => true,
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
            _ if is_dec_digit_start(ch) => self.consume_decimal_literal(),
            _ if ch.is_whitespace() => self.consume_whitespace(),
            _ if is_identifier_start(ch) => self.consume_identifier(),

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
            _ => TokenKind::Symbol,
        }
    }

    /// Consume a decimal (base 10) literal (such as `123.45` or `123`)
    fn consume_decimal_literal(&mut self) -> TokenKind {
        // Assume we are lexing the string `123.45`

        // After this we'll have `123`
        self.skip_while(is_dec_digit_continue);

        if self.peek() == Some('.') {
            // Now `123.`
            self.advance();

            // Now `123.45`
            self.skip_while(is_dec_digit_continue);

            match f64::from_str(self.token_slice()) {
                Ok(_) => TokenKind::FloatLiteral,
                Err(e) => {
                    self.add_diagnostic(
                        Diagnostic::new_error("unable to parse text as a 64-bit floating point")
                            .with_code("E0002")
                            .with_label(Label::new_primary(self.token_span()))
                    );

                    self.add_diagnostic(Diagnostic::new_note(format!("{}", e)));

                    TokenKind::Error
                },
            }
        } else {
            // We're already at the end of the literal so just parse it
            match i64::from_str_radix(self.token_slice(), 10) {
                Ok(_) => TokenKind::IntLiteral,
                Err(e) => {
                    self.add_diagnostic(
                        Diagnostic::new_error("unable to parse text as a signed 64-bit integer")
                            .with_code("E0003")
                            .with_label(Label::new_primary(self.token_span()))
                    );

                    self.add_diagnostic(Diagnostic::new_note(format!("{}", e)));

                    TokenKind::Error
                },
            }
        }
    }

    /// Consume everything until we hit a newline sequence
    fn consume_comment(&mut self) -> TokenKind {
        self.skip_while(|ch| ch != '\n' && ch != '\r');

        TokenKind::Comment
    }

    /// Consume an identifier
    fn consume_identifier(&mut self) -> TokenKind {
        self.skip_while(is_identifier_continue);

        let slice = self.token_slice();
        match slice {
            _ if slice.eq_ignore_ascii_case("true") => TokenKind::True,
            _ if slice.eq_ignore_ascii_case("yes") => TokenKind::Yes,
            _ if slice.eq_ignore_ascii_case("false") => TokenKind::False,
            _ if slice.eq_ignore_ascii_case("no") => TokenKind::No,
            _ => TokenKind::Identifier,
        }
    }

    /// Consume whitespace
    fn consume_whitespace(&mut self) -> TokenKind {
        // TODO: Skip whitespace until a newline seq so we have different
        //       tokens for newline and indentation levels
        //       which, potentially, allows for more lint rules
        //       ex: must indent with spaces
        self.skip_while(char::is_whitespace);
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
mod tests {
    use mltt_span::Files;
    use language_reporting::{
        Severity,
    };
    use super::*;

    /// A handy macro to give us a nice syntax for declaring test cases
    ///
    /// This was inspired by the tests in the LALRPOP lexer
    macro_rules! test {
        ($src:expr, $($span:expr => $token:expr,)*) => {{
            let _ = simple_logger::init(); // ignore failure

            let mut files = Files::new();
            let file_id = files.add("test", $src);
            let lexed_tokens: Vec<_> = Lexer::new(&files[file_id])
                .map(|result| result)
                .collect();
            let expected_tokens = vec![$({
                let (kind, slice) = $token;
                let start = ByteIndex::from($span.find("~").unwrap());
                let end = ByteIndex::from($span.rfind("~").unwrap()) + ByteSize::from(1);
                let span = FileSpan::new(file_id, start, end);
                Token { kind, slice, span }
            }),*];

            assert_eq!(lexed_tokens, expected_tokens);
        }};
    }

    #[test]
    fn data() {
        test! {
            "wowza",
            "~~~~~" => (TokenKind::Identifier, "wowza"),
        }

        test! {
            " wowza ",
            "~      " => (TokenKind::Whitespace, " "),
            " ~~~~~ " => (TokenKind::Identifier, "wowza"),
            "      ~" => (TokenKind::Whitespace, " "),
        }

        test! {
            "hello: world",
            "~~~~~       " => (TokenKind::Identifier, "hello"),
            "     ~      " => (TokenKind::Colon, ":"),
            "      ~     " => (TokenKind::Whitespace, " "),
            "       ~~~~~" => (TokenKind::Identifier, "world"),
        }

        test! {
            "hello: ^world",
            "~~~~~        " => (TokenKind::Identifier, "hello"),
            "     ~       " => (TokenKind::Colon, ":"),
            "      ~      " => (TokenKind::Whitespace, " "),
            "       ~     " => (TokenKind::Caret, "^"),
            "        ~~~~~" => (TokenKind::Identifier, "world"),
        }
    }

    #[test]
    fn remove_inherited_property() {
        test! {
            "-SomeProperty:",
            "~             " => (TokenKind::Symbol, "-"),
            " ~~~~~~~~~~~~ " => (TokenKind::Identifier, "SomeProperty"),
            "             ~" => (TokenKind::Colon, ":"),
        }
    }

    #[test]
    fn dash_symbol_followed_by_whitespace() {
        test! {
            "- Foobar",
            "~       " => (TokenKind::Symbol, "-"),
            " ~      " => (TokenKind::Whitespace, " "),
            "  ~~~~~~" => (TokenKind::Identifier, "Foobar"),
        }
    }

    #[test]
    fn numbers() {
        test! {
            "0",
            "~" => (TokenKind::IntLiteral, "0"),
        }

        test! {
            "1",
            "~" => (TokenKind::IntLiteral, "1"),
        }

        test! {
            "123.45",
            "~~~~~~" => (TokenKind::FloatLiteral, "123.45"),
        }

        test! {
            "-123",
            "~~~~" => (TokenKind::IntLiteral, "-123"),
        }

        test! {
            "-123.45",
            "~~~~~~~" => (TokenKind::FloatLiteral, "-123.45"),
        }
    }

    #[test]
    fn symbols() {
        // single-char symbols
        test! {
            "~!@:^#",
            "~     " => (TokenKind::Tilde, "~"),
            " ~    " => (TokenKind::Bang, "!"),
            "  ~   " => (TokenKind::At, "@"),
            "   ~  " => (TokenKind::Colon, ":"),
            "    ~ " => (TokenKind::Caret, "^"),
            "     ~" => (TokenKind::Comment, "#"),
        }

        // composite symbols
        test! {
            "|| &&",
            "~~   " => (TokenKind::LogicalOr, "||"),
            "  ~  " => (TokenKind::Whitespace, " "),
            "   ~~" => (TokenKind::LogicalAnd, "&&"),
        }
    }

    #[test]
    fn newlines() {
        test! {
            "\n",
            "~" => (TokenKind::Eol, "\n"),
        }

        test! {
            "\r\n",
            "~~" => (TokenKind::Eol, "\r\n"),
        }

        test! {
            "^Foo:\n",
            "~      " => (TokenKind::Caret, "^"),
            " ~~~   " => (TokenKind::Identifier, "Foo"),
            "    ~  " => (TokenKind::Colon, ":"),
            "     ~ " => (TokenKind::Eol, "\n"),
        }

        test! {
            "^Foo:\r\n",
            "~        " => (TokenKind::Caret, "^"),
            " ~~~     " => (TokenKind::Identifier, "Foo"),
            "    ~    " => (TokenKind::Colon, ":"),
            "     ~~  " => (TokenKind::Eol, "\r\n"),
        }

        // sequential newlines yield separate tokens
        test! {
            "\n\n",
            "~   " => (TokenKind::Eol, "\n"),
            " ~  " => (TokenKind::Eol, "\n"),
        }

        // sequential newlines yield separate tokens
        test! {
            "\r\n\r\n",
            "~~      " => (TokenKind::Eol, "\r\n"),
            "  ~~    " => (TokenKind::Eol, "\r\n"),
        }
    }

    #[test]
    fn carriage_return_only_yields_error_token_kind() {
        // invalid newline sequence yields an error token
        test! {
            "\r",
            "~ " => (TokenKind::Error, "\r"),
        }
    }

    #[test]
    fn carriage_return_only_has_warning_diagnostic() {
        // Arrange
        let mut files = Files::new();
        let file_id = files.add("test", "\r");

        // Act
        let mut lexer = Lexer::new(&files[file_id]);
        let tokens: Vec<_> = lexer.by_ref().collect();

        // Assert
        assert_eq!(tokens.len(), 1);
        let token = &tokens[0];
        assert_eq!(token.kind, TokenKind::Error);

        let diags = lexer.take_diagnostics();
        assert_eq!(diags.len(), 1);

        let diag = &diags[0];
        assert_eq!(diag.severity, Severity::Warning);
        assert_eq!(diag.code, Some("W0004".into()));
        assert!(diag.message.contains("invalid newline sequence"));
    }

    #[test]
    fn bang_identifier() {
        test! {
            "!foo-bar",
            "~       " => (TokenKind::Bang, "!"),
            " ~~~~~~~" => (TokenKind::Identifier, "foo-bar"),
        }
    }

    #[test]
    fn idents_symbols_and_comments() {
        test! {
            "a@",
            "~ " => (TokenKind::Identifier, "a"),
            " ~" => (TokenKind::At, "@"),
        }

        test! {
            "a@b",
            "~  " => (TokenKind::Identifier, "a"),
            " ~ " => (TokenKind::At, "@"),
            "  ~" => (TokenKind::Identifier, "b"),
        }

        test! {
            "a@b:",
            "~   " => (TokenKind::Identifier, "a"),
            " ~  " => (TokenKind::At, "@"),
            "  ~ " => (TokenKind::Identifier, "b"),
            "   ~" => (TokenKind::Colon, ":"),
        }

        test! {
            "a@b: c # d",
            "~         " => (TokenKind::Identifier, "a"),
            " ~        " => (TokenKind::At, "@"),
            "  ~       " => (TokenKind::Identifier, "b"),
            "   ~      " => (TokenKind::Colon, ":"),
            "    ~     " => (TokenKind::Whitespace, " "),
            "     ~    " => (TokenKind::Identifier, "c"),
            "      ~   " => (TokenKind::Whitespace, " "),
            "       ~~~" => (TokenKind::Comment, "# d"),
        }
    }

    #[test]
    fn keyword_true() {
        test! {
            "true",
            "~~~~" => (TokenKind::True, "true"),
        }

        test! {
            "True",
            "~~~~" => (TokenKind::True, "True"),
        }
    }

    #[test]
    fn keyword_false() {
        test! {
            "false",
            "~~~~~" => (TokenKind::False, "false"),
        }

        test! {
            "False",
            "~~~~~" => (TokenKind::False, "False"),
        }
    }

    #[test]
    fn keyword_no() {
        test! {
            "no",
            "~~" => (TokenKind::No, "no"),
        }

        test! {
            "No",
            "~~" => (TokenKind::No, "No"),
        }
    }

    #[test]
    fn keyword_yes() {
        test! {
            "yes",
            "~~~" => (TokenKind::Yes, "yes"),
        }

        test! {
            "Yes",
            "~~~" => (TokenKind::Yes, "Yes"),
        }
    }

    #[test]
    fn op_and() {
        test! {
            "&&",
            "~~" => (TokenKind::LogicalAnd, "&&"),
        }
    }

    #[test]
    fn op_or() {
        test! {
            "||",
            "~~" => (TokenKind::LogicalOr, "||"),
        }
    }

    #[test]
    fn comment() {
        // basic
        test! {
            "# hello, friends",
            "~~~~~~~~~~~~~~~~" => (TokenKind::Comment, "# hello, friends"),
        }

        // multiple # symbols only produce a single comment
        test! {
            "# hello, # friends",
            "~~~~~~~~~~~~~~~~~~" => (TokenKind::Comment, "# hello, # friends"),
        }

        // no preceding whitespace
        test! {
            "#hello, friends",
            "~~~~~~~~~~~~~~~" => (TokenKind::Comment, "#hello, friends"),
        }

        // multiple spaces between "end" of comment and eol are included in span
        test! {
            "# this is a comment    ",
            "~~~~~~~~~~~~~~~~~~~~~~~" => (TokenKind::Comment, "# this is a comment    "),
        }

        // spaces in comment are included
        test! {
            "# this is a comment with multiple       spaces",
            "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~" => (TokenKind::Comment, "# this is a comment with multiple       spaces"),
        }

        // comment with large whitespace and multiple # symbols
        test! {
            "# this is a comment with multiple     # spaces",
            "~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~" => (TokenKind::Comment, "# this is a comment with multiple     # spaces"),
        }
    }
}
