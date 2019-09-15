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

use std::{
    collections::VecDeque,
    sync::mpsc::Sender,
    io::{self, Read as _, Write as _},
    fmt,
};

use serde::{
    Serialize,
    Deserialize,
};

use oraide_actor::{
    TaskId,
    Actor,
    Symbol,
    QueryRequest,
    QueryResponse,
};

mod language_server_ctx;
pub mod types;

pub use language_server_ctx::{
    LanguageServerCtx,
    LanguageServerCtxStorage,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "lowercase")]
pub enum LspMessage {
    Initialize {
        id: usize,
        params: languageserver_types::InitializeParams,
    },

    Initialized,

    #[serde(rename = "textDocument/didOpen")]
    TextDocDidOpen {
        params: languageserver_types::DidOpenTextDocumentParams,
    },

    #[serde(rename = "textDocument/didChange")]
    TextDocDidChange {
        params: languageserver_types::DidChangeTextDocumentParams,
    },

    #[serde(rename = "textDocument/hover")]
    TextDocHover {
        id: usize,
        params: languageserver_types::TextDocumentPositionParams,
    },

    #[serde(rename = "textDocument/definition")]
    TextDocDefinition {
        id: usize,
        params: languageserver_types::TextDocumentPositionParams,
    },

    #[serde(rename = "textDocument/documentSymbol")]
    TextDocSymbols {
        id: usize,
        params: languageserver_types::DocumentSymbolParams,
    },

    #[serde(rename = "$/cancelRequest")]
    CancelRequest {
        params: languageserver_types::CancelParams,
    },
}

/// The LSP service is split into two parts:
///   * The server, which handles incoming requests from the IDE
///   * The responder, which sends out results when they're ready
/// The server sends messages *to* the task manager for work that
/// needs to be done. The responder receives messages *from* the
/// task manager for work that has been accomplished.
pub struct LspResponder;

impl Actor for LspResponder {
    type Input = QueryResponse;

    // Map our `QueryResponse` type to `languageserver_types` types
    fn on_new_messages(&mut self, messages: &mut VecDeque<Self::Input>) {
        match messages.pop_front().unwrap() {
            QueryResponse::Nothing { task_id } => send_response(task_id, ()),
            QueryResponse::AckInitialize { task_id } => {
                let result = languageserver_types::InitializeResult {
                    capabilities: languageserver_types::ServerCapabilities {
                        text_document_sync: Some(
                            languageserver_types::TextDocumentSyncCapability::Kind(
                                languageserver_types::TextDocumentSyncKind::Incremental,
                            ),
                        ),
                        hover_provider: Some(true),
                        completion_provider: None,
                        signature_help_provider: None,
                        definition_provider: Some(true),
                        type_definition_provider: None,
                        implementation_provider: None,
                        references_provider: Some(false),
                        document_highlight_provider: None,
                        document_symbol_provider: true.into(),
                        workspace_symbol_provider: None,
                        code_action_provider: None,
                        code_lens_provider: None,
                        document_formatting_provider: None,
                        document_range_formatting_provider: None,
                        document_on_type_formatting_provider: None,
                        rename_provider: None,
                        color_provider: None,
                        folding_range_provider: None,
                        execute_command_provider: None,
                        workspace: None,
                    },
                };

                send_response(task_id, result);
            },
            QueryResponse::HoverData { task_id, data } => {
                send_response(task_id, languageserver_types::Hover {
                    contents: languageserver_types::HoverContents::Scalar(
                        languageserver_types::MarkedString::from_markdown(data),
                    ),
                    range: None,
                });
            },
            QueryResponse::Definition { task_id, ranged_file_position } => {
                match ranged_file_position {
                    None => send_response(task_id, Option::<languageserver_types::Location>::None),
                    Some(ranged_file_position) => {
                        let start_pos = {
                            let start = ranged_file_position.range.start;
                            languageserver_types::Position::new(start.line_idx as u64, start.character_idx as u64)
                        };

                        let end_exclusive_pos = {
                            let end_exclusive = ranged_file_position.range.end_exclusive;
                            languageserver_types::Position::new(end_exclusive.line_idx as u64, end_exclusive.character_idx as u64)
                        };


                        let range = languageserver_types::Range::new(
                            start_pos,
                            end_exclusive_pos
                        );

                        let location = languageserver_types::Location::new(
                            ranged_file_position.file_url,
                            range,
                        );

                        send_response(task_id, location);
                    },
                }
            },
            QueryResponse::DocumentSymbols { task_id, symbols } => {
                fn oraide_sym_to_doc_sym(sym: Symbol) -> languageserver_types::DocumentSymbol {
                    languageserver_types::DocumentSymbol {
                        name: sym.name,
                        detail: sym.detail,
                        kind: languageserver_types::SymbolKind::Object,
                        range: sym.range.clone().into(),
                        selection_range: sym.range.into(),
                        deprecated: None,
                        children: sym.children.map(|children|
                            children.into_iter()
                                .map(oraide_sym_to_doc_sym)
                                .collect()
                        ),
                    }
                }

                let symbols: Vec<_> = symbols.into_iter()
                    .map(oraide_sym_to_doc_sym)
                    .collect();

                send_response(task_id, symbols);
            },
        }
    }
}

/// A wrapper for responses back to the client from the server.
/// These must follow the JSON 2.0 RPC spec.
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse<T> {
    jsonrpc: String,
    id: usize,
    result: T,
}

impl<T> JsonRpcResponse<T> {
    pub fn new(id: usize, result: T) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result,
        }
    }
}

/// A wrapper for notifications to the client from the server.
/// These must follow the JSON 2.0 RPC spec.
#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcNotification<T> {
    jsonrpc: String,
    method: String,
    params: T,
}

impl<T> JsonRpcNotification<T> {
    pub fn new(method: String, params: T) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            method,
            params,
        }
    }
}

/// Helper function to send a result back to the client
fn send_response<T: Serialize + fmt::Debug>(task_id: TaskId, response: T) {
    let response = JsonRpcResponse::new(task_id, response);
    let response_string = match serde_json::to_string(&response) {
        Ok(s) => s,
        Err(e) => {
            log::error!("Could not serialize data for response `{:?}`: {}", response, e);
            return;
        },
    };

    print!("Content-Length: {}\r\n\r\n{}", response_string.len(), response_string);
    let _ = io::stdout().flush();
}

/// Helper function to send a notification to the client
fn send_notification<T: Serialize + fmt::Debug>(method: String, notice: T) {
    let notice = JsonRpcNotification::new(method, notice);
    let notice_string = serde_json::to_string(&notice).unwrap();

    print!("Content-Length: {}\r\n\r\n{}", notice_string.len(), notice_string);
    let _ = io::stdout().flush();
}

/// The workhorse function for handling incoming requests from an LSP client.
/// This will take instructions from STDIN sent by the client and send them to
/// the appropriate system.
pub fn lsp_serve(send_to_query_channel: Sender<QueryRequest>) {
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let content_length_items: Vec<&str> = input.split(' ').collect();
                if content_length_items[0] == "Content-Length:" {
                    let num_bytes: usize = content_length_items[1].trim().parse().unwrap();
                    let mut buffer = vec![0u8; num_bytes + 2];
                    let _ = io::stdin().read_exact(&mut buffer);

                    let buffer_string = String::from_utf8(buffer).unwrap();

                    let message = serde_json::from_str::<LspMessage>(&buffer_string);

                    match message {
                        Ok(LspMessage::Initialize { id: task_id, params }) => {
                            let _ = send_to_query_channel.send(
                                QueryRequest::Initialize {
                                    task_id,
                                    workspace_root_url: params.root_uri,
                                }
                            );
                        },
                        Ok(LspMessage::Initialized) => {
                            // intentionally empty, nothing to do
                        },
                        Ok(LspMessage::TextDocDidOpen { params }) => {
                            let _ = send_to_query_channel.send(
                                QueryRequest::FileOpened {
                                    file_url: params.text_document.uri,
                                    file_text: params.text_document.text,
                                }
                            );
                        },
                        Ok(LspMessage::TextDocDidChange { params }) => {
                            let _ = send_to_query_channel.send(QueryRequest::FileChanged {
                                file_url: params.text_document.uri,
                                changes: params
                                    .content_changes
                                    .into_iter()
                                    .map(|x| (
                                        // Since we are using `Incremental` text document sync
                                        // we will *always* have a `range`
                                        x.range.unwrap(),
                                        x.text,
                                    ))
                                    .collect(),
                            });
                        },
                        Ok(LspMessage::TextDocHover { id: task_id, params }) => {
                            let _ = send_to_query_channel.send(QueryRequest::HoverAtPosition {
                                task_id,
                                file_url: params.text_document.uri,
                                file_pos: params.position,
                            });
                        },
                        Ok(LspMessage::TextDocDefinition { id: task_id, params }) => {
                            let _ = send_to_query_channel.send(QueryRequest::GoToDefinition {
                                task_id,
                                file_url: params.text_document.uri,
                                file_pos: params.position,
                            });
                        },
                        Ok(LspMessage::TextDocSymbols { id: task_id, params }) => {
                            let _ = send_to_query_channel.send(QueryRequest::FileSymbols {
                                task_id,
                                file_url: params.text_document.uri,
                            });
                        },
                        Ok(LspMessage::CancelRequest { .. }) => {}
                        Err(_e) => {},
                    }
                }
            },
            _ => unimplemented!("lsp_serve io::stdin().read_line _ match arm"),
        }
    }
}