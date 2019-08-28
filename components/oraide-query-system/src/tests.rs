use oraide_span::{
    ByteIndex,
};

use crate::OraideDatabase;

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
fn token_spanning_byte_index() {
    // Arrange
    let mut db = OraideDatabase::default();
    let text = "E1:\n\tTooltip:\n\t\tName: Standard Infantry\n";
    let file_id = db.add_file("test-file", text.clone());

    // Act
    let opt_actual_token = db.token_spanning_byte_index(file_id, ByteIndex::from(24));

    // Assert
    assert!(opt_actual_token.is_some());

    let actual_token = opt_actual_token.unwrap();

    let expected_token_key_text = "Standard";
    assert_eq!(
        actual_token.text(text).unwrap(),
        expected_token_key_text,
        "the `{}` token should have been returned", expected_token_key_text
    );
}

#[test]
fn node_spanning_byte_index() {
    // Arrange
    let mut db = OraideDatabase::default();
    let text = "E1:\n\tTooltip:\n\t\tName: Standard Infantry\n";
    let file_id = db.add_file("test-file", text.clone());

    // Act
    let opt_actual_node = db.node_spanning_byte_index(file_id, ByteIndex::from(24));

    // Assert
    assert!(opt_actual_node.is_some());

    let actual_node = opt_actual_node.unwrap();

    let expected_node_key_text = "Name";
    assert_eq!(
        actual_node.key_text(text).unwrap(),
        expected_node_key_text,
        "the `{}` node should have been returned", expected_node_key_text
    );
}

#[test]
fn position_to_byte_index() {
    // Arrange
    let mut db = OraideDatabase::default();
    let text = "E1:\n\tTooltip:\n\t\tName: Standard Infantry\n";
    let file_id = db.add_file("test-file", text.clone());

    // Act
    let opt_actual_byte_idx = db.position_to_byte_index(file_id, languageserver_types::Position {
        line: 2,
        character: 10,
    });

    // Assert
    let expected_idx = byte_index_of_nth_char_in_str(2, 'a', text);
    assert!(opt_actual_byte_idx.is_some());

    let actual_byte_idx = opt_actual_byte_idx.unwrap();
    assert_eq!(actual_byte_idx, expected_idx);
}