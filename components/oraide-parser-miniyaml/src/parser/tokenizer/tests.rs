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

use oraide_span::{
    ByteIndex,
    FileId,
    FileSpan,
};

use crate::*;

/// Compute the `ByteIndex` of the `n`-th (1-based) `ch` in `s`
///
/// # Example
/// ```rust
/// let idx_of_2nd_n = byte_index_of_nth_char_in_str(2, 'n', "Name: McKenna");
/// ```
fn byte_index_of_nth_char_in_str(n: usize, ch: char, s: &str) -> ByteIndex {
    assert!(n > 0, "n={}", n);
    assert!(n < s.len(), "n={} < s.len()={}", n, s.len());

    let idx = s
        .match_indices(ch)
        .nth(n - 1) // `nth` is 0-based (https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.nth)
        .map(|(idx, _)| idx)
        .expect(&format!(
            "TEST LOGIC ERROR: {}({}, {}, {:?})",
            stringify!(byte_index_of_nth_char_in_str),
            n, ch, s
        ));

    ByteIndex::from(idx)
}

#[test]
fn token_text() {
    // Arrange
    let input = "foo bar baz";

    let tokens = vec![
        ( // `b`
            byte_index_of_nth_char_in_str(1, 'b', input),
            byte_index_of_nth_char_in_str(1, 'b', input) + 1.into(),
        ),
        ( // `ba`
            byte_index_of_nth_char_in_str(1, 'b', input),
            byte_index_of_nth_char_in_str(1, 'a', input) + 1.into(),
        ),
        ( // `bar`
            byte_index_of_nth_char_in_str(1, 'b', input),
            byte_index_of_nth_char_in_str(1, 'r', input) + 1.into(),
        ),
        ( // `ar`
            byte_index_of_nth_char_in_str(1, 'a', input),
            byte_index_of_nth_char_in_str(1, 'r', input) + 1.into(),
        ),
        ( // `r`
            byte_index_of_nth_char_in_str(1, 'r', input),
            byte_index_of_nth_char_in_str(1, 'r', input) + 1.into(),
        ),
    ].into_iter()
        .map(|(s, e)| FileSpan::new(FileId(0), s, e))
        .map(|span| Token { kind: TokenKind::Error, span, })
        .collect::<Vec<_>>();

    // Act
    let opt_texts = tokens.into_iter()
        .map(|token| token.text(input))
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(opt_texts, vec![
        Some("b"),
        Some("ba"),
        Some("bar"),
        Some( "ar"),
        Some(  "r"),
    ]);
}

#[test]
fn token_text_invalid_start() {
    // Arrange
    let input = "ÿ";
    let token = {
        let start = 0;
        let end_exclusive = 1; // 2 would make this test pass
        let span = FileSpan::new(FileId(0), 0, 1);
        Token {
            kind: TokenKind::Error, // doesn't matter for this test
            span,
        }
    };

    // Act
    let opt_text = token.text(input);

    // Assert
    assert_eq!(opt_text, None);
}

#[test]
fn token_text_invalid_end_exclusive() {
    // Arrange
    let input = "ÿ";
    let token = {
        let start = 1; // 0 would make this test pass
        let end_exclusive = 2;
        let span = FileSpan::new(FileId(0), 0, 1);
        Token {
            kind: TokenKind::Error, // doesn't matter for this test
            span,
        }
    };

    // Act
    let opt_text = token.text(input);

    // Assert
    assert_eq!(opt_text, None);

}

#[test]
#[should_panic]
fn tokenizer_token_span_with_invalid_span_start_panics() {
    // Arrange
    let tokenizer = {
        let mut t = Tokenizer::new(FileId(0), "ÿ");

        // `1` is not a char boundary so when `t.token_span()` is invoked,
        // it will fail an assertion and panic
        std::mem::swap(&mut t.token_start, &mut 1.into());

        t
    };

    // Act & Assert
    let _tokens = tokenizer.collect::<Vec<_>>();
}

#[test]
#[should_panic]
fn tokenizer_token_span_with_invalid_span_end_exclusive_panics() {
    // Arrange
    let tokenizer = {
        let mut t = Tokenizer::new(FileId(0), "ÿ");

        // `1` is not a char boundary so when `t.token_span()` is invoked,
        // it will fail an assertion and panic
        std::mem::swap(&mut t.token_end_exclusive, &mut 1.into());

        t
    };

    // Act & Assert
    let _tokens = tokenizer.collect::<Vec<_>>();
}