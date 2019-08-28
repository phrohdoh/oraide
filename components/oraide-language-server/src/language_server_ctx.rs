use {
    std::{
        fs::File,
        path::PathBuf,
        io::Read as _,
    },
    oraide_span::{
        FileId,
    },
    oraide_actor::{
        Position,
    },
    oraide_parser_miniyaml::{
        ParserCtx,
        TokenKind,
    },
    crate::{
        types,
    },
    url::Url,
};

#[salsa::query_group(LanguageServerCtxStorage)]
pub trait LanguageServerCtx: ParserCtx {
    #[salsa::input]
    fn workspace_root(&self) -> Option<PathBuf>;

    fn type_data(&self) -> Option<Vec<types::TraitDetail>>;

    fn documentation_lines_for_type_data(
        &self,
        type_name: String,
    ) -> Option<Vec<String>>;

    fn documentation_for_position_in_file_path(
        &self,
        file_path: String,
        position: Position,
    ) -> Option<String>;

    fn documentation_for_position_in_file(
        &self,
        file_id: FileId,
        position: Position,
    ) -> Option<String>;

    fn definition_position_in_file_path(
        &self,
        file_path: String,
        position: Position,
    ) -> Option<(Url, Position, Position)>;

    fn definition_position_in_file(
        &self,
        file_id: FileId,
        position: Position,
    ) -> Option<(Url, Position, Position)>;
}

fn type_data(db: &impl LanguageServerCtx) -> Option<Vec<types::TraitDetail>> {
    let type_data_json_file_path = {
        // Q: Why does this function return an `Option<_>`?
        // A: Since language servers require clients to be of any use
        //    we've built a reference implementation for VSCode.
        //    The VSCode client will send `Some(root_uri)` _iff_ a directory is
        //    opened (instead of just a file, for example) which the server
        //    will set `workspace_root` to.
        //    If `workspace_root` is `None` we won't be able to load type-data
        //    since we can not derive a path to the type-data file from the
        //    workspace's root.
        //    In this case returning `None` is the only logical thing to do.
        let mut path = match db.workspace_root() {
            Some(path) => path,
            _ => {
                eprintln!("Failed to determine type-data file path due to unset workspace root");
                return None;
            },
        };

        path.push(".oraide");
        path.push("type-data.json");
        path
    };

    let type_data = {
        let mut s = String::new();
        let mut f = match File::open(&type_data_json_file_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!(
                    "Failed to open file `{}`: {:?}",
                    type_data_json_file_path.display(),
                    e,
                );
                return None;
            },
        };

        let _ = f.read_to_string(&mut s).ok()?;
        match serde_json::from_str(&s) {
            Ok(de) => Some(de),
            Err(e) => {
                eprintln!("Failed to deserialize JSON[1] due to error[2]");
                eprintln!("[1]: {}", s);
                eprintln!("[2]: {:?}", e);
                return None;
            },
        }
    };

    type_data
}

fn documentation_lines_for_type_data(
    db: &impl LanguageServerCtx,
    type_name: String,
) -> Option<Vec<String>> {
    let type_details = match db.type_data() {
        Some(details) => details,
        _ => {
            eprintln!("No type-data in database");
            return None;
        },
    };

    let item_detail = match type_details.into_iter().find(|td| td.name == type_name) {
        Some(detail) => detail,
        _ => {
            eprintln!("No type-data with name[1] found in database");
            eprintln!("[1]: {:#?}", type_name);
            return None;
        },
    };

    item_detail.doc_lines
}

fn documentation_for_position_in_file_path(
    db: &impl LanguageServerCtx,
    file_path: String,
    position: Position,
) -> Option<String> {
    let file_id = match db.file_id_of_file_path(file_path.clone()) {
        Some(id) => id,
        _ => {
            log::error!("No `FileId` found for file path `{}`", file_path);
            return None;
        },
    };

    db.documentation_for_position_in_file(file_id, position)
}

fn documentation_for_position_in_file(
    db: &impl LanguageServerCtx,
    file_id: FileId,
    position: Position,
) -> Option<String> {
    let file_text = match db.file_text(file_id) {
        Some(text) => text,
        _ => {
            eprintln!("No text in database for `FileId`[1]");
            eprintln!("[1]: {}", file_id.0);
            return None;
        },
    };

    let byte_index = match db.convert_position_to_byte_index(file_id, position) {
        Some(idx) => idx,
        _ => {
            eprintln!("Failed to convert `Position`[1] in `FileId`[2] to a `ByteIndex`");
            eprintln!("[1]: {:?}", position);
            eprintln!("[2]: {}", file_id.0);
            return None;
        },
    };

    let token = match db.token_spanning_byte_index_in_file(file_id, byte_index) {
        Some(token) => token,
        _ => {
            eprintln!("Failed to get `Token` spanning `ByteIndex`[1] in `FileId`[2]");
            eprintln!("[1]: {:?}", byte_index);
            eprintln!("[2]: {}", file_id.0);
            return None;
        },
    };

    let token_text = match token.text(&file_text) {
        Some(text) => text,
        _ => {
            eprintln!("Failed to determine text for `Token`[1] in contents of `FileId`[2]");
            eprintln!("[1]: {:?}", token);
            eprintln!("[2]: {}", file_id.0);
            return None;
        },
    };

    let doc_lines = match db.documentation_lines_for_type_data(token_text.to_owned()) {
        Some(lines) => lines,
        _ => {
            eprintln!("Failed to determine documentation lines from type-data for text[1]");
            eprintln!("[1]: {:#?}", token_text);
            return None;
        },
    };

    let joined_doc_lines = doc_lines.join("\n");
    Some(joined_doc_lines)
}

fn definition_position_in_file_path(
    db: &impl LanguageServerCtx,
    file_path: String,
    position: Position,
) -> Option<(Url, Position, Position)> {
    let file_id = match db.file_id_of_file_path(file_path.clone()) {
        Some(id) => id,
        _ => {
            log::error!("No `FileId` found for file `{}`", file_path);
            return None;
        },
    };

    db.definition_position_in_file(file_id, position)
}

fn definition_position_in_file(
    db: &impl LanguageServerCtx,
    file_id: FileId,
    position: Position,
) -> Option<(Url, Position, Position)> {
    let file_text = db.file_text(file_id)?;
    let byte_index = db.convert_position_to_byte_index(file_id, position)?;

    // Get the entire `Node` so we can grab multiple `Token`s if necessary
    let node = db.node_spanning_byte_index_in_file(file_id, byte_index)?;

    let tokens = node.into_tokens();
    let mut tokens_iter = tokens.iter();

    // Find the token that the user requested the definition of
    let token_idx = tokens_iter.position(|token| token.span.contains(byte_index))?;
    let token = &tokens[token_idx];
    let token_text = token.text(&file_text)?;

    let text_to_search_for = match tokens.get(token_idx - 1) {
        // If the text in the document is `^Foobar`, for example, then include
        // the `^` in the query (for OpenRA's `Inherits`).
        Some(prev_token) if prev_token.kind == TokenKind::Caret => format!("^{}", token_text),
        _ => token_text.into(),
    };

    let file_ids = db.all_file_ids();

    for f_id in file_ids {
        if let Some(node) = db.top_level_node_by_key_in_file(f_id, text_to_search_for.clone()) {
            let key_span = match node.key_span() {
                Some(span) => span,
                _ => continue,
            };

            let (start_pos, end_exclusive_pos) = match db.convert_file_span_to_2_positions(key_span) {
                Some((s, e)) => (s, e),
                _ => continue,
            };

            let file_path = db.file_path(f_id)?;

            use std::str::FromStr as _;
            let file_url = Url::from_str(&file_path).ok()?;

            return Some((file_url, start_pos, end_exclusive_pos));
        }
    }

    None
}