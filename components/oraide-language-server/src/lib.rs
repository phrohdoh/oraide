// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

mod server;
mod lsp;
mod work_pool;
mod dispatch;
mod actions;
mod concurrency;
mod context;

pub use server::{
    MsgReader,
    StdinMsgReader,
    StdoutOutput,
    LangServerService,
};

pub fn run_server() -> i32 {
    let svc = LangServerService::new(
        Box::new(StdinMsgReader),
        StdoutOutput::new(),
    );

    svc.run()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
