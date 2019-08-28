// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    collections::VecDeque,
    sync::mpsc::Sender,
    thread,
};

use salsa::{
    ParallelDatabase,
    Snapshot,
};

use oraide_actor::{
    RangedFilePosition,
    Actor,
    QueryRequest,
    QueryResponse,
};

use oraide_parser_miniyaml::{
    FilesCtx,
    FilesCtxExt,
    FilesCtxStorage,
    TextFilesCtx,
    TextFilesCtxExt,
    TextFilesCtxStorage,
    ParserCtx,
    ParserCtxStorage,
};

use oraide_language_server::{
    LanguageServerCtx,
    LanguageServerCtxStorage,
};

/// Entrypoint into MiniYaml parsing
///
/// Contains inputs and memoized computation results
///
/// # Example
/// ```rust
/// # use oraide_query_system::OraideDatabase;
/// use oraide_parser_miniyaml::{
///     ParserCtx,
///     ParserCtxExt,
///     Tree,
/// };
///
/// let mut db = OraideDatabase::default();
/// let file_id = db.add_text_file("example.yaml", "Hello:\n");
/// let tree: Tree = db.file_tree(file_id);
/// ```
#[salsa::database(
    FilesCtxStorage,
    TextFilesCtxStorage,
    ParserCtxStorage,
    LanguageServerCtxStorage,
)]
pub struct OraideDatabase {
    rt: salsa::Runtime<Self>,
}

impl salsa::Database for OraideDatabase {
    fn salsa_runtime(&self) -> &salsa::Runtime<Self> {
        &self.rt
    }
}

impl Default for OraideDatabase {
    fn default() -> Self {
        let mut db = Self {
            rt: salsa::Runtime::default(),
        };

        db.init();
        db
    }
}

impl FilesCtx for OraideDatabase {}          // _: salsa::Database
impl TextFilesCtx for OraideDatabase {}      // _: FilesCtx
impl ParserCtx for OraideDatabase {}         // _: TextFilesCtx
impl LanguageServerCtx for OraideDatabase {} // _: ParserCtx

impl FilesCtxExt for OraideDatabase {}       // _: FilesCtx
impl TextFilesCtxExt for OraideDatabase {}   // _: TextFilesCtx

impl ParallelDatabase for OraideDatabase {
    fn snapshot(&self) -> Snapshot<Self> {
        Snapshot::new(Self {
            rt: self.rt.snapshot(self),
        })
    }
}

pub struct QuerySystem {
    /// The channel used to send messages to a client
    send_channel: Sender<QueryResponse>,
    db: OraideDatabase,
    needs_run_diags: bool,
}

impl Actor for QuerySystem {
    type Input = QueryRequest;

    fn on_new_messages(&mut self, messages: &mut VecDeque<Self::Input>) {
        // Find the last message that will mutate the server state.
        let opt_last_mutating_idx = messages.iter()
            .rposition(QueryRequest::will_mutate_server_state);

        // Up until that point we need to process *only* mutating messages.
        if let Some(last_mutating_idx) = opt_last_mutating_idx {
            for message in messages.drain(0..=last_mutating_idx) {
                if message.will_mutate_server_state() {
                    self.process_message(message);
                }
            }

            // After each mutation we need to perform diagnostics checking
            self.needs_run_diags = true;
        }

        // All the mutations are processed, now process the next non-mutation.
        if let Some(message) = messages.pop_front() {
            assert!(!message.will_mutate_server_state());
            self.process_message(message);
        }
    }
}

impl QuerySystem {
    pub fn new(send_channel: Sender<QueryResponse>) -> Self {
        Self {
            send_channel,
            db: OraideDatabase::default(),
            needs_run_diags: false,
        }
    }

    fn process_message(&mut self, message: QueryRequest) {
        match message {
            QueryRequest::Initialize { task_id, workspace_root_url } => {
                let chan = self.send_channel.clone();
                send(chan, QueryResponse::AckInitialize { task_id });

                if let Some(workspace_root_path) = workspace_root_url.and_then(|url| url.to_file_path().ok()) {
                    self.db.set_workspace_root(workspace_root_path.into());
                }
            },
            QueryRequest::HoverAtPosition { task_id, file_url, file_pos } => {
                thread::spawn({
                    let db = self.db.snapshot();
                    let chan = self.send_channel.clone();

                    move || {
                        match db.hover_with_file_name(file_url.to_string(), file_pos.into()) {
                            Some(md) => send(chan, QueryResponse::HoverData {
                                task_id, 
                                data: md.0,
                            }),
                            _ => send(chan, QueryResponse::HoverData {
                                task_id,
                                data: "<no results>".into(),
                            })
                        }
                    }
                });
            },
            QueryRequest::GoToDefinition { task_id, file_url, file_pos } => {
                thread::spawn({
                    let db = self.db.snapshot();
                    let chan = self.send_channel.clone();

                    move || {
                        match db.definition_with_file_name(file_url.to_string(), file_pos.into()) {
                            Some((file_url, start_pos, end_exclusive_pos)) => send(chan, QueryResponse::Definition {
                                task_id,
                                ranged_file_position: RangedFilePosition::new_from_components(
                                    file_url,
                                    start_pos,
                                    end_exclusive_pos,
                                ).into(),
                            }),
                            _ => send(chan, QueryResponse::Definition {
                                task_id,
                                ranged_file_position: None,
                            }),
                        }
                    }
                });
            },
            QueryRequest::FileOpened { file_url, file_text } => {
                // TODO: How will we handle duplicates?
                let _ = self.db.add_file(file_url.as_str(), file_text);
            },
            QueryRequest::FileChanged { file_url, changes } => {
                let file_id = self.db.file_name_to_file_id(file_url.to_string()).unwrap();
                let mut current_contents = self.db.file_text(file_id);

                for change in changes {
                    let start_offset = self.db.position_to_byte_index(
                        file_id,
                        change.0.start.into(),
                    ).unwrap();

                    let end_offset = self.db.position_to_byte_index(
                        file_id,
                        change.0.end.into(),
                    ).unwrap();

                    current_contents.drain(start_offset.to_usize()..end_offset.to_usize());
                    current_contents.insert_str(start_offset.to_usize(), &change.1);
                }

                self.db.set_file_text(file_id, current_contents);
            },
        }
    }
}

fn send(channel: Sender<QueryResponse>, message: QueryResponse) {
    if let Err(err) = channel.send(message) {
        log::error!("internal error: {}", err);
    }
}

#[cfg(test)]
mod tests;