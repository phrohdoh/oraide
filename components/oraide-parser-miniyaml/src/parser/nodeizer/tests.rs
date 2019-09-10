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

use unindent::unindent;
use pretty_assertions::assert_eq;

use oraide_span::{
    FileId,
    FileSpan,
};

use crate::{
    Token,
    TokenKind,
    Tokenizer,
    Node,
    Nodeizer,
};

macro_rules! token {
    ($file_id:tt, $kind:tt, $span_start:tt .. $span_end:tt) => {
        Token {
            kind: TokenKind::$kind,
            span: FileSpan::new($file_id, $span_start, $span_end),
        }
    };
}

#[test]
fn key_only_without_key_terminator() {
    // Arrange
    let src = unindent("
        Packages:
            foo
            bar: baz
    ");

    let file_id = FileId(0);

    let lexer = Tokenizer::new(file_id, &src);
    let tokens = lexer.collect::<Vec<_>>();

    let nodeizer = Nodeizer::new(tokens.into_iter());

    // Act
    let actual_nodes = nodeizer.collect::<Vec<_>>();

    // Assert
    let actual_node_count = actual_nodes.len();

    let expected_nodes = vec![
        Node {
            indentation_token: None,
            key_tokens: vec![
                token!(file_id, Identifier, 0..8),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, 8..9),
            ),
            value_tokens: vec![],
            comment_token: None,
        },
        Node {
            indentation_token: Some(
                token!(file_id, Whitespace, 10..14),
            ),
            key_tokens: vec![
                token!(file_id, Identifier, 14..17),
            ],
            key_terminator_token: None,
            value_tokens: vec![],
            comment_token: None,
        },
        Node {
            indentation_token: Some(
                token!(file_id, Whitespace, 18..22),
            ),
            key_tokens: vec![
                token!(file_id, Identifier, 22..25),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, 25..26),
            ),
            value_tokens: vec![
                token!(file_id, Whitespace, 26..27),
                token!(file_id, Identifier, 27..30),
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
    // Arrange
    let src = unindent("
        ^BasePlayer@Wookie: # 0
            AlwaysVisible:  # 1

        Player:
            Inherits: ^BasePlayer@Wookie
    ");

    let file_id = FileId(0);

    let lexer = Tokenizer::new(file_id, &src);
    let tokens = lexer.collect::<Vec<_>>();

    let nodeizer = Nodeizer::new(tokens.into_iter());

    // Act
    let actual_nodes = nodeizer.collect::<Vec<_>>();

    // Assert
    let actual_node_count = actual_nodes.len();

    let expected_nodes = vec![
        Node {
            indentation_token: None,
            key_tokens: vec![
                token!(file_id, Caret, 0..1),
                token!(file_id, Identifier, 1..11),
                token!(file_id, At, 11..12),
                token!(file_id, Identifier, 12..18),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, 18..19),
            ),
            value_tokens: vec![
                token!(file_id, Whitespace, 19..20),
            ],
            comment_token: Some(
                token!(file_id, Comment, 20..23),
            ),
        },
        Node {
            indentation_token: Some(
                token!(file_id, Whitespace, 24..28),
            ),
            key_tokens: vec![
                token!(file_id, Identifier, 28..41),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, 41..42),
            ),
            value_tokens: vec![
                token!(file_id, Whitespace, 42..44),
            ],
            comment_token: Some(
                token!(file_id, Comment, 44..47),
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
                token!(file_id, Identifier, 49..55),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, 55..56),
            ),
            value_tokens: vec![],
            comment_token: None,
        },
        Node {
            indentation_token: Some(
                token!(file_id, Whitespace, 57..61),
            ),
            key_tokens: vec![
                token!(file_id, Identifier, 61..69),
            ],
            key_terminator_token: Some(
                token!(file_id, Colon, 69..70),
            ),
            value_tokens: vec![
                token!(file_id, Whitespace, 70..71),
                token!(file_id, Caret, 71..72),
                token!(file_id, Identifier, 72..82),
                token!(file_id, At, 82..83),
                token!(file_id, Identifier, 83..89),
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
fn key_text() {
    // Arrange
    let src = unindent("
        ^BasePlayer@Wookie: # 0
            AlwaysVisible:  # 1

        Player:
            Inherits: ^BasePlayer@Wookie
    ");

    let file_id = FileId(0);

    let lexer = Tokenizer::new(file_id, &src);
    let tokens = lexer.collect::<Vec<_>>();

    let nodeizer = Nodeizer::new(tokens.into_iter());

    // Act
    let actual_key_texts = nodeizer
        .map(|node| node.key_text(&src))
        .collect::<Vec<_>>();

    // Assert
    let expected_key_texts = vec![
        Some("^BasePlayer@Wookie"),
        Some("AlwaysVisible"),
        None,
        Some("Player"),
        Some("Inherits"),
    ];

    assert_eq!(actual_key_texts, expected_key_texts);
}