use unindent::unindent;
use pretty_assertions::assert_eq;

use mltt_span::{
    Files,
    FileSpan,
};

use crate::{
    Lexer,
    Parser,
    TokenKind,
    Token,
    Node,
};

macro_rules! token {
    ($file_id:tt, $kind:tt, $slice:expr, $span_start:tt .. $span_end:tt) => {
        Token {
            kind: TokenKind::$kind,
            slice: $slice,
            span: FileSpan::new($file_id, $span_start, $span_end),
        }
    };
}

#[test]
fn key_only_without_key_terminator() {
    let _ = env_logger::try_init(); // ignore failure

    // Arrange
    let src = unindent("
        Packages:
            foo
            bar: baz
    ");

    let mut files = Files::new();
    let file_id = files.add("test", src);
    let file = &files[file_id];

    let lexer = Lexer::new(file);
    let tokens = lexer.collect::<Vec<_>>();

    let parser = Parser::new(file_id, tokens.into_iter());

    // Act
    let actual_nodes = parser.collect::<Vec<_>>();

    // Assert
    let actual_node_count = actual_nodes.len();

    let expected_nodes = vec![
        Node {
            indentation_token: None,
            key_tokens: vec![
                token!(file_id, Identifier, "Packages", 0..8),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, ":", 8..9),
            ),
            value_tokens: vec![],
            comment_token: None,
        },
        Node {
            indentation_token: Some(
                token!(file_id, Whitespace, "    ", 10..14),
            ),
            key_tokens: vec![
                token!(file_id, Identifier, "foo", 14..17),
            ],
            key_terminator_token: None,
            value_tokens: vec![],
            comment_token: None,
        },
        Node {
            indentation_token: Some(
                token!(file_id, Whitespace, "    ", 18..22),
            ),
            key_tokens: vec![
                token!(file_id, Identifier, "bar", 22..25),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, ":", 25..26),
            ),
            value_tokens: vec![
                token!(file_id, Whitespace, " ", 26..27),
                token!(file_id, Identifier, "baz", 27..30),
            ],
            comment_token: None,
        },
    ];

    let expected_node_count = expected_nodes.len();

    assert_eq!(
        actual_nodes.len(),
        expected_node_count,
        "Expected {} node(s), but got {}",
        expected_node_count,
        actual_node_count,
    );

    assert_eq!(actual_nodes, expected_nodes);
}

#[test]
fn general_test() {
    let _ = env_logger::try_init(); // ignore failure

    // Arrange
    let src = unindent("
        ^BasePlayer@Wookie: # 0
            AlwaysVisible:  # 1

        Player:
            Inherits: ^BasePlayer@Wookie
    ");

    let mut files = Files::new();
    let file_id = files.add("test", src);
    let file = &files[file_id];

    let lexer = Lexer::new(file);
    let tokens = lexer.collect::<Vec<_>>();

    let parser = Parser::new(file_id, tokens.into_iter());

    // Act
    let actual_nodes = parser.collect::<Vec<_>>();

    // Assert
    let actual_node_count = actual_nodes.len();

    let expected_nodes = vec![
        Node {
            indentation_token: None,
            key_tokens: vec![
                token!(file_id, Caret, "^", 0..1),
                token!(file_id, Identifier, "BasePlayer", 1..11),
                token!(file_id, At, "@", 11..12),
                token!(file_id, Identifier, "Wookie", 12..18),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, ":", 18..19),
            ),
            value_tokens: vec![
                token!(file_id, Whitespace, " ", 19..20),
            ],
            comment_token: Some(
                token!(file_id, Comment, "# 0", 20..23),
            ),
        },
        Node {
            indentation_token: Some(
                token!(file_id, Whitespace, "    ", 24..28),
            ),
            key_tokens: vec![
                token!(file_id, Identifier, "AlwaysVisible", 28..41),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, ":", 41..42),
            ),
            value_tokens: vec![
                token!(file_id, Whitespace, "  ", 42..44),
            ],
            comment_token: Some(
                token!(file_id, Comment, "# 1", 44..47),
            ),
        },
        Node {
            indentation_token: None,
            key_tokens: vec![],
            key_terminator_token: None,
            value_tokens: vec![],
            comment_token: None,
        },
        Node {
            indentation_token: None,
            key_tokens: vec![
                token!(file_id, Identifier, "Player", 49..55),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, ":", 55..56),
            ),
            value_tokens: vec![],
            comment_token: None,
        },
        Node {
            indentation_token: Some(
                token!(file_id, Whitespace, "    ", 57..61),
            ),
            key_tokens: vec![
                token!(file_id, Identifier, "Inherits", 61..69),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, ":", 69..70),
            ),
            value_tokens: vec![
                token!(file_id, Whitespace, " ", 70..71),
                token!(file_id, Caret, "^", 71..72),
                token!(file_id, Identifier, "BasePlayer", 72..82),
                token!(file_id, At, "@", 82..83),
                token!(file_id, Identifier, "Wookie", 83..89),
            ],
            comment_token: None,
        },
    ];

    let expected_node_count = expected_nodes.len();

    assert_eq!(
        actual_nodes.len(),
        expected_node_count,
        "Expected {} node(s), but got {}",
        expected_node_count,
        actual_node_count,
    );

    assert_eq!(actual_nodes, expected_nodes);
}