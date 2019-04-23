// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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