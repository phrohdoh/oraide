pub use lsp_types::notification::{
    Initialized,
};

use crate::{
    server::{
        BlockingNotificationAction,
        Output,
    },
    context::{
        InitContext,
    },
};

impl BlockingNotificationAction for Initialized {
    fn handle<O: Output>(_: Self::Params, _: &mut InitContext, _: O) -> Result<(), ()> {
        log::trace!("Client has let the server know it has initialized");
        Ok(())
    }
}