use std::{
    io::{
        self,
        Read as _,
    },
    path::{
        PathBuf,
    },
    fs::{
        File,
    },
};

use oraide_span::{
    ByteIndex,
    FileId,
};

use crate::{
    LangServerCtx,
    Markdown,
    Position,
    lang_server::types,
};

pub(crate) fn type_data(db: &impl LangServerCtx) -> Option<Vec<types::TraitDetail>> {
    let file_path = db.type_data_json_file_path()?;
    let mut file_content = String::new();
    let mut f = File::open(&file_path).expect(&format!("Failed to open `{}`", file_path.display()));
    f.read_to_string(&mut file_content).unwrap();
    serde_json::de::from_str(&file_content).unwrap()
}

pub(crate) fn type_data_json_file_path(db: &impl LangServerCtx) -> Option<PathBuf> {
    // Q: Why does this function return `Option<_>`?
    // A: Since language servers require clients of some sort to be useful
    //    we've built a reference implementation for VSCode.
    //    The VSCode client will send `Some(root_uri)` _iff_ a directory is
    //    opened (instead of just a file, for example).
    //    The server will set `dot_dir_path` using this `root_uri`.
    //    So if a file is opened, instead of a directory, `dot_dir_path` will
    //    not be set which means we won't have a path to a type-data file
    //    in which case returning `Option::None` is the only logical thing to do.

    let mut path = db.dot_dir_path()?;
    path.push("type-data.json");
    path.into()
}

pub(crate) fn doc_lines_for_trait(db: &impl LangServerCtx, trait_name: String) -> Option<Vec<String>> {
    // See https://github.com/Phrohdoh/OpenRA/tree/oraide-util-extract-type-data
    let trait_details = db.type_data()?;
    let detail = trait_details.into_iter().find(|td| td.name == trait_name)?;
    detail.doc_lines
}

pub(crate) fn position_to_byte_index(db: &impl LangServerCtx, file_id: FileId, pos: Position) -> Option<ByteIndex> {
    let line_offsets = db.line_offsets(file_id);

    let line_num = pos.line_idx + 1;
    if line_offsets.len() < line_num {
        return None;
    }

    let line_start = line_offsets[pos.line_idx];
    Some(ByteIndex::from(line_start + pos.character_idx))
}

pub(crate) fn hover_with_file_name(db: &impl LangServerCtx, file_name: String, pos: Position) -> Option<Markdown> {
    let file_id = match db.file_name_to_file_id(file_name.to_owned()) {
        Some(fid) => fid,
        _ => {
            eprintln!("No file id found for file name `{}`", file_name);
            return None;
        },
    };

    db.hover_with_file_id(file_id, pos)
}

pub(crate) fn hover_with_file_id(db: &impl LangServerCtx, file_id: FileId, pos: Position) -> Option<Markdown> {
    let file_text = db.file_text(file_id);
    let byte_index = db.position_to_byte_index(file_id, pos)?;
    let token = db.token_spanning_byte_index(file_id, byte_index)?;
    let token_text = token.text(&file_text)?;

    let trimmed_token_text = token_text.trim();

    // Returning an empty string to the client isn't helpful, so return a
    // `None` here that we'll convert into something indicating 'no results'
    // in `QuerySystem::process_message`.
    if trimmed_token_text.is_empty() {
        return None;
    }

    let doc_lines = db.doc_lines_for_trait(trimmed_token_text.to_owned())?;

    let joined_doc_string = doc_lines.join("\n");

    return Some(joined_doc_string.into());
}