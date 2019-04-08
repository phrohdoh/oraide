pub use lsp_types::notification::{
    Initialized,
};

use crate::{
    lsp::{
        Notification,
    },
    server::{
        BlockingNotificationAction,
        Output,
    },
    context::{
        InitContext,
    },
};

impl BlockingNotificationAction for Initialized {
    fn handle<O: Output>(
        _params: Self::Params,
        ctx: &mut InitContext,
        out: O,
    ) -> Result<(), ()> {
        log::trace!("Client has let the server know it has initialized");
        Ok(())
    }
}