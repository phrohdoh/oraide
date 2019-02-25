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
            indentation_tokens: vec![],
            key_tokens: vec![
                token!(file_id, Caret, "^", 0..1),
                token!(file_id, Identifier, "BasePlayer", 1..11),
                token!(file_id, At, "@", 11..12),
                token!(file_id, Identifier, "Wookie", 12..18),
            ],
            // NOTE: The `:` is skipped so span byte indices will not be consecutive
            value_tokens: vec![
                token!(file_id, Whitespace, " ", 19..20),
            ],
            comment_token: Some(
                token!(file_id, Comment, "# 0", 20..23),
            ),
        },
        Node {
            indentation_tokens: vec![
                token!(file_id, Whitespace, "    ", 24..28),
            ],
            key_tokens: vec![
                token!(file_id, Identifier, "AlwaysVisible", 28..41),
            ],
            // NOTE: The `:` is skipped so span byte indices will not be consecutive
            value_tokens: vec![
                token!(file_id, Whitespace, "  ", 42..44),
            ],
            comment_token: Some(
                token!(file_id, Comment, "# 1", 44..47),
            ),
        },
        Node {
            indentation_tokens: vec![],
            key_tokens: vec![],
            value_tokens: vec![],
            comment_token: None,
        },
        Node {
            indentation_tokens: vec![],
            key_tokens: vec![
                token!(file_id, Identifier, "Player", 49..55),
            ],
            // NOTE: The `:` is skipped so span byte indices will not be consecutive
            value_tokens: vec![],
            comment_token: None,
        },
        Node {
            indentation_tokens: vec![
                token!(file_id, Whitespace, "    ", 57..61),
            ],
            key_tokens: vec![
                token!(file_id, Identifier, "Inherits", 61..69),
            ],
            // NOTE: The `:` is skipped so span byte indices will not be consecutive
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