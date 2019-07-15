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
    Actor,
    QueryRequest,
    QueryResponse,
};

use oraide_parser_miniyaml::{
    ParserCtxExt,
    ParserCtxStorage,
};

mod lang_server;
pub use lang_server::{
    LangServerCtx,
    LangServerCtxStorage,
    Markdown,
};

mod query_definitions;

/// Position in a text document expressed as zero-based line and character offset.
/// A position is between two characters like an 'insert' cursor in a editor.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone, Default, Hash)]
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

/// Entrypoint into MiniYaml parsing
///
/// Contains inputs and memoized computation results
///
/// # Example
/// ```rust
/// # use oraide_query_system::Database;
/// use oraide_parser_miniyaml::{
///     ParserCtx,
///     ParserCtxExt,
///     Tree,
/// };
///
/// let mut db = Database::default();
/// let file_id = db.add_file("example.yaml", "Hello:\n");
/// let tree: Tree = db.file_tree(file_id);
/// ```
#[salsa::database(ParserCtxStorage, LangServerCtxStorage)]
pub struct Database {
    rt: salsa::Runtime<Self>,
}

impl salsa::Database for Database {
    fn salsa_runtime(&self) -> &salsa::Runtime<Self> {
        &self.rt
    }
}

impl Default for Database {
    fn default() -> Self {
        let mut db = Self {
            rt: salsa::Runtime::default(),
        };

        db.init();
        db
    }
}

impl ParserCtxExt for Database {}

impl ParallelDatabase for Database {
    fn snapshot(&self) -> Snapshot<Self> {
        Snapshot::new(Self {
            rt: self.rt.snapshot(self),
        })
    }
}

pub struct QuerySystem {
    /// The channel used to send messages to a client
    send_channel: Sender<QueryResponse>,
    db: Database,
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
            db: Database::default(),
            needs_run_diags: false,
        }
    }

    fn process_message(&mut self, message: QueryRequest) {
        match message {
            QueryRequest::Initialize { task_id, workspace_root_url } => {
                let chan = self.send_channel.clone();
                send(chan, QueryResponse::AckInitialize { task_id });

                if let Some(workspace_root_path) = workspace_root_url.and_then(|url| url.to_file_path().ok()) {
                    let dot_dir_path = workspace_root_path.join(".oraide");
                    self.db.set_dot_dir_path(dot_dir_path.into());
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
            QueryRequest::FileOpened { file_url, file_text } => {
                // TODO: How will we handle duplicates?
                let _ = self.db.add_file(file_url.as_str(), file_text);
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