use {
    oraide_span::{
        FileId,
        FileSpan,
        ByteIndex,
        Location,
    },
    oraide_actor::{
        Position,
    },
    crate::{
        FilesCtx,
        TextFilesCtx,
    },
};

// --- FilesCtx queries ---

pub(crate) fn file_id_of_file_path(db: &impl FilesCtx, file_path: String) -> Option<FileId> {
    db.all_file_ids()
        .into_iter()
        .find(|file_id| db.file_path(*file_id).as_ref() == Some(&file_path))
}

// --- TextFilesCtx queries ---

pub(crate) fn line_start_offsets(
    db: &impl TextFilesCtx,
    file_id: FileId,
) -> Option<Vec<usize>> {
    let file_text = &db.file_text(file_id)?;
    let mut acc = 0;

    file_text.lines()
        .map(|line_text| {
            let line_start = acc;
            acc += line_text.len();

            if file_text[acc..].starts_with("\r\n") {
                acc += 2;
            } else if file_text[acc..].starts_with("\n") {
                acc += 1;
            }

            line_start
        })
        .chain(std::iter::once(file_text.len()))
        .collect::<Vec<_>>()
        .into()
}

pub(crate) fn convert_file_span_to_2_positions(
    db: &impl TextFilesCtx,
    span: FileSpan,
) -> Option<(Position, Position)> {
    let file_id = span.source();

    let start = {
        let location = db.convert_byte_index_to_location(file_id, span.start())?;
        Position::new(
            location.line_number - 1,
            location.column_number - 1,
        )
    };

    let end_exclusive = {
        let location = db.convert_byte_index_to_location(file_id, span.end_exclusive())?;
        Position::new(
            location.line_number - 1,
            location.column_number - 1,
        )
    };

    Some((start, end_exclusive))
}


pub(crate) fn convert_byte_index_to_location(
    db: &impl TextFilesCtx,
    file_id: FileId,
    byte_index: ByteIndex,
) -> Option<Location> {
    let line_start_offsets = db.line_start_offsets(file_id)?;
    let byte_index = byte_index.to_usize();

    match line_start_offsets.binary_search(&byte_index) {
        Ok(line_idx) => {
            // We found the start of the line directly
            Location::new(line_idx + 1, 1)
        },
        Err(next_line_num) => {
            let line_idx = next_line_num - 1;

            // We found something in the middle
            let line_start_idx = line_start_offsets[line_idx];

            // Count utf-8 chars to determine column
            let file_text = db.file_text(file_id)?;
            let column = file_text[line_start_idx..byte_index].chars().count();

            Location::new(next_line_num, column)
        },
    }.into()
}

pub(crate) fn convert_position_to_byte_index(
    db: &impl TextFilesCtx,
    file_id: FileId,
    position: Position,
) -> Option<ByteIndex> {
    let line_start_offsets = db.line_start_offsets(file_id)?;
    let line_start_idx = line_start_offsets.get(position.line_idx)?;
    ByteIndex::from(line_start_idx + position.character_idx).into()
}