use serde::{
    Serialize,
    Deserialize,
};

use languageserver_types::{
    Url,
};

/// Position in a text document expressed as zero-based line and character offset.
/// A position is between two characters like an 'insert' cursor in a editor.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Hash, Deserialize, Serialize)]
pub struct Position {
    /// 0-based
    pub line_idx: usize,

    /// 0-based
    pub character_idx: usize,
}

impl Position {
    pub fn new(line_idx: usize, character_idx: usize) -> Self {
        Self {
            line_idx,
            character_idx,
        }
    }
}

impl From<languageserver_types::Position> for Position {
    fn from(pos: languageserver_types::Position) -> Self {
        Self {
            line_idx: pos.line as usize,
            character_idx: pos.character as usize,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash, Deserialize, Serialize)]
pub struct Range<T> {
    pub start: T,
    pub end_exclusive: T,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Hash)]
pub struct RangedFilePosition {
    pub file_url: Url,
    pub range: Range<Position>,
}

impl RangedFilePosition {
    pub fn new(file_url: Url, range: Range<Position>) -> Self {
        Self {
            file_url,
            range,
        }
    }

    pub fn new_from_components(
        file_url: Url,
        range_start: Position,
        range_end_exclusive: Position,
    ) -> Self {
        Self {
            file_url,
            range: Range {
                start: range_start,
                end_exclusive: range_end_exclusive,
            }
        }
    }
}