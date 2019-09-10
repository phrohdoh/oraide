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

// use std::sync::mpsc::{channel, Receiver, RecvError, Sender, TryRecvError};
use oraide_actor::{spawn_actor, /* Actor, QueryResponse */};
use oraide_language_server::{lsp_serve, LspResponder};
use oraide_query_system::{QuerySystem};

pub fn ide() {
    let lsp_responder = spawn_actor(LspResponder);
    let query_system = spawn_actor(QuerySystem::new(lsp_responder.channel));

    lsp_serve(query_system.channel);
}