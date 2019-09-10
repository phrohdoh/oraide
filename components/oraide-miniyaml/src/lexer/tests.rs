// This file is part of oraide.  See <https://github.com/Phrohdoh/oraide>.
// 
// oraide is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3
// as published by the Free Software Foundation.
// 
// oraide is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with oraide.  If not, see <https://www.gnu.org/licenses/>.

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
        let _ = env_logger::try_init(); // ignore failure

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
fn identifier_start_then_number_yields_single_identifier_token() {
    test! {
        "T01",
        "~~~" => (TokenKind::Identifier, "T01"),
    }
}

#[test]
fn identifier_allows_dashes() {
    test! {
        "my-identifier",
        "~~~~~~~~~~~~~" => (TokenKind::Identifier, "my-identifier"),
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
fn consume_whitespace_until_eol() {
    test! {
        "    \t\n\t",
        "~~~~~     " => (TokenKind::Whitespace, "    \t"),
        "     ~    " => (TokenKind::Eol, "\n"),
        "      ~   " => (TokenKind::Whitespace, "\t"),
    }

    test! {
        "    \t\r\n\t",
        "~~~~~      " => (TokenKind::Whitespace, "    \t"),
        "     ~~    " => (TokenKind::Eol, "\r\n"),
        "       ~   " => (TokenKind::Whitespace, "\t"),
    }

    // comments also stop at eol
    test! {
        "hello # test  \n   ",
        "~~~~~ # test  \n   " => (TokenKind::Identifier, "hello"),
        "     ~             " => (TokenKind::Whitespace, " "),
        "      ~~~~~~~~\n   " => (TokenKind::Comment, "# test  "),
        "              ~    " => (TokenKind::Eol, "\n"),
        "               ~~~ " => (TokenKind::Whitespace, "   "),
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

mod props {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn does_not_crash(src in "\\PC*") {
            let _ = env_logger::try_init(); // ignore failure

            let mut files = Files::new();
            let file_id = files.add("test", src);
            let _lexed_tokens: Vec<_> = Lexer::new(&files[file_id]).collect();
        }

        #[test]
        fn consume_symbol_given_letters_should_return_tokenkind_error_variant_and_add_diag_bug(src in "[a-zA-Z]+") {
            let _ = env_logger::try_init(); // ignore failure
            log::trace!("{:?}", src);

            // Arrange
            let mut files = Files::new();
            let file_id = files.add("test", src);
            let file = &files[file_id];
            let mut lexer = Lexer::new(&file);

            let expected_token_kind = TokenKind::Error;

            // Act
            let actual_token_kind = lexer.consume_symbol();

            // Assert
            assert_eq!(actual_token_kind, expected_token_kind);

            let diags = lexer.take_diagnostics();
            assert!(!diags.is_empty());

            let diag = &diags[0];
            assert_eq!(diag.severity, Severity::Bug);
            assert!(diag.message.contains("consume_symbol"));
            assert!(diag.message.contains("invalid Lexer state"));
            assert!(diag.message.contains("expected next charater to be a symbol"));
        }

        #[test]
        fn consume_symbol_given_digits_should_return_tokenkind_error_variant_and_add_diag_bug(src in "[0-9]+") {
            let _ = env_logger::try_init(); // ignore failure
            log::trace!("{:?}", src);

            // Arrange
            let mut files = Files::new();
            let file_id = files.add("test", src);
            let file = &files[file_id];
            let mut lexer = Lexer::new(&file);

            let expected_token_kind = TokenKind::Error;

            // Act
            let actual_token_kind = lexer.consume_symbol();

            // Assert
            assert_eq!(actual_token_kind, expected_token_kind);

            let diags = lexer.take_diagnostics();
            assert!(!diags.is_empty());

            let diag = &diags[0];
            assert_eq!(diag.severity, Severity::Bug);
            assert!(diag.message.contains("consume_symbol"));
            assert!(diag.message.contains("invalid Lexer state"));
            assert!(diag.message.contains("expected next charater to be a symbol"));
        }

        #[test]
        fn collect_with_letters_only_yields_identifier(
            src in "[a-zA-Z]+"
        ) {
            let _ = env_logger::try_init(); // ignore failure
            log::trace!("{:?}", src);

            // Arrange
            let mut files = Files::new();
            let file_id = files.add("test", src);
            let file = &files[file_id];
            let lexer = Lexer::new(&file);

            let expected_token_kind = TokenKind::Identifier;

            // Act
            let tokens = lexer.collect::<Vec<_>>();

            // Assert
            assert_eq!(tokens.len(), 1);
            let token = &tokens[0];

            assert_eq!(token.kind, expected_token_kind);
        }

        #[test]
        fn collect_with_leading_digits_trailing_any_valid_identifier_char_yields_identifier(
            src in "[0-9][a-zA-Z-._]+"
        ) {
            let _ = env_logger::try_init(); // ignore failure
            log::trace!("{:?}", src);

            // Arrange
            let mut files = Files::new();
            let file_id = files.add("test", src);
            let file = &files[file_id];
            let mut lexer = Lexer::new(&file);

            let expected_token_kind = TokenKind::Identifier;

            // Act
            let tokens = lexer.by_ref().collect::<Vec<_>>();

            // Assert
            assert_eq!(tokens.len(), 1, "{:?}", tokens);

            let token = &tokens[0];
            assert_eq!(token.kind, expected_token_kind);
        }

        #[test]
        fn consume_identifier(
            src in "[a-zA-Z][a-zA-Z0-9-._]*"
        ) {
            let _ = env_logger::try_init(); // ignore failure
            log::trace!("{:?}", src);

            // Arrange
            let mut files = Files::new();
            let file_id = files.add("test", src);
            let file = &files[file_id];
            let mut lexer = Lexer::new(&file);

            let expected_token_kind = TokenKind::Identifier;

            // Act
            let tokens = lexer.by_ref().collect::<Vec<_>>();

            // Assert
            assert_eq!(tokens.len(), 1, "{:?}", tokens);

            let token = &tokens[0];
            assert_eq!(token.kind, expected_token_kind);
        }

        #[test]
        fn consume_decimal_literal_given_integer_works(
            src in "[0-9-][0-9]{1,6}"
        ) {
            let _ = env_logger::try_init(); // ignore failure
            log::trace!("{:?}", src);

            // Arrange
            let mut files = Files::new();
            let file_id = files.add("test", src.clone());
            let file = &files[file_id];
            let mut lexer = Lexer::new(&file);

            let expected_token_kind = TokenKind::IntLiteral;

            // Act
            let tokens = lexer.by_ref().collect::<Vec<_>>();

            // Assert
            assert_eq!(tokens.len(), 1);

            let tok = &tokens[0];
            assert_eq!(tok.kind, expected_token_kind);
            assert_eq!(tok.slice, src);
        }

        #[test]
        fn consume_decimal_literal_given_float_works(
            src in "[0-9-][0-9]{1,6}[.][0-9]{1,8}"
        ) {
            let _ = env_logger::try_init(); // ignore failure
            log::trace!("{:?}", src);

            // Arrange
            let mut files = Files::new();
            let file_id = files.add("test", src.clone());
            let file = &files[file_id];
            let mut lexer = Lexer::new(&file);

            let expected_token_kind = TokenKind::FloatLiteral;

            // Act
            let tokens = lexer.by_ref().collect::<Vec<_>>();

            // Assert
            assert_eq!(tokens.len(), 1);

            let tok = &tokens[0];
            assert_eq!(tok.kind, expected_token_kind);
            assert_eq!(tok.slice, src);
        }
    }
}