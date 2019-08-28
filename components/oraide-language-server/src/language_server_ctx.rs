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
    },
    crate::{
        types,
    },
};

#[salsa::query_group(LanguageServerCtxStorage)]
pub trait LanguageServerCtx: ParserCtx {
    #[salsa::input]
    fn workspace_root(&self) -> Option<PathBuf>;

    fn type_data(&self) -> Option<Vec<types::TraitDetail>>;

    fn documentation_lines_for_item(&self, item_name: String) -> Option<Vec<String>>;

    fn documentation_for_item_with_file_path(
        &self,
        source_file_path: String,
        position: Position,
    ) -> Option<String>;

    fn documentation_for_top_level_node_in_file(
        &self,
        file_id: FileId,
        position: Position,
    ) -> Option<String>;
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
        let mut path = db.workspace_root()?;
        path.push(".oraide");
        path.push("type-data.json");
        path
    };

    let type_data = {
        let mut s = String::new();
        let mut f = match File::open(&type_data_json_file_path) {
            Ok(f) => f,
            Err(e) => {
                log::warn!(
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
                log::warn!(
                    "Failed to deserialize JSON: {:?}",
                    e,
                );
                return None;
            },
        }
    };

    type_data
}

fn documentation_lines_for_item(db: &impl LanguageServerCtx, item_name: String) -> Option<Vec<String>> {
    let type_details = db.type_data()?;
    let item_detail = type_details.into_iter().find(|td| td.name == item_name)?;
    item_detail.doc_lines
}

fn documentation_for_item_with_file_path(
    db: &impl LanguageServerCtx,
    source_file_path: String,
    position: Position,
) -> Option<String> {
    let file_id = match db.file_id_of_file_path(source_file_path.clone()) {
        Some(id) => id,
        _ => {
            log::error!("No `FileId` found for file path `{}`", source_file_path);
            return None;
        },
    };

    db.documentation_for_top_level_node_in_file(file_id, position)
}

fn documentation_for_top_level_node_in_file(
    db: &impl LanguageServerCtx,
    file_id: FileId,
    position: Position,
) -> Option<String> {
    let file_text = db.file_text(file_id)?;
    let byte_index = db.convert_position_to_byte_index(file_id, position)?;
    let token = db.token_spanning_byte_index_in_file(file_id, byte_index)?;
    let token_text = token.text(&file_text)?;
    let doc_lines = db.documentation_lines_for_item(token_text.to_owned())?;
    let joined_doc_lines = doc_lines.join("\n");
    Some(joined_doc_lines)
}