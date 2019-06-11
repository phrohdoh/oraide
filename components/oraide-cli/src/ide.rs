// use std::sync::mpsc::{channel, Receiver, RecvError, Sender, TryRecvError};
use oraide_actor::{spawn_actor, /* Actor, QueryResponse */};
use oraide_language_server::{lsp_serve, LspResponder};
use oraide_query_system::{QuerySystem};

pub fn ide() {
    let lsp_responder = spawn_actor(LspResponder);
    let query_system = spawn_actor(QuerySystem::new(lsp_responder.channel));

    lsp_serve(query_system.channel);
}