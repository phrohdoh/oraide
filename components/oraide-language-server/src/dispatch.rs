// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use lsp_types::request::HoverRequest;
use std::{
    sync::mpsc,
    thread,
    time::Duration,
};

use jsonrpc_core::{
    ErrorCode,
};

use lsp_types::request::Request as LspRequest;

use crate::{
    lsp::Request,
    server::{
        Output,
        Response as ServerResponse,
        ResponseError,
    },
    work_pool::{
        self,
        WorkDescription,
    },
    concurrency::{
        ConcurrentJob,
        JobToken,
    },
    context::{
        InitContext,
    },
};

/// Timeout duration for request responses.  By default an LSP client request
/// not responded to after this duration will return a fallback response.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_millis(1_500);

macro_rules! define_dispatch_request_enum {
    ($($req_type:ident),*$(,)*) => {
        pub(crate) enum DispatchRequest {
            $(
                $req_type(Request<$req_type>),
            )*
        }

        $(
            impl From<Request<$req_type>> for DispatchRequest {
                fn from(req: Request<$req_type>) -> Self {
                    DispatchRequest::$req_type(req)
                }
            }
        )*

        impl DispatchRequest {
            fn handle<O: Output>(self, ctx: InitContext, out: &O) {
                match self {
                $(
                    DispatchRequest::$req_type(req) => {
                        let Request { id, params, received, .. } = req;
                        let timeout = $req_type::timeout();

                        let rx = work_pool::receive_from_thread(move || {
                            // Checking the timeout here can prevent starting
                            // expensive work that has already timed out due to
                            // previous long-running requests.
                            // Note: We're doing this here, on the threadpool,
                            // as pool scheduling may incur a further delay.
                            if received.elapsed() >= timeout {
                                $req_type::fallback_response()
                            } else {
                                $req_type::handle(ctx, params)
                            }
                        }, WorkDescription($req_type::METHOD));

                        match rx.recv_timeout(timeout).unwrap_or_else(|_| $req_type::fallback_response()) {
                            Ok(resp) => resp.send(id, out),
                            Err(ResponseError::Empty) => out.send_failure_message(id, ErrorCode::InternalError, "An unknown error occurred"),
                            Err(ResponseError::Message(code, msg)) => out.send_failure_message(id, code, msg),
                        }
                    },
                )*
                }
            }
        }
    };
}

define_dispatch_request_enum!(
    HoverRequest,
);

/// Provides the ability to dispatch requests to a worker thread that will
/// handle the requests sequentially, without blocking the input channel.
/// Requests dispatched this way are automatically timed out and avoid
/// processing if they have already timed out before starting.
pub(crate) struct Dispatcher {
    sender: mpsc::Sender<(DispatchRequest, InitContext, JobToken)>,
}

impl Dispatcher {
    /// Create a new `Dispatcher` starting a new thread and channel
    pub(crate) fn new<O: Output>(out: O) -> Self {
        let (sender, rx) = mpsc::channel::<(DispatchRequest, InitContext, JobToken)>();

        thread::Builder::new()
            .name("dispatch-worker".into())
            .spawn(move || {
                while let Ok((req, ctx, token)) = rx.recv() {
                    req.handle(ctx, &out);
                    drop(token);
                }
            })
            .unwrap();

        Self { sender }
    }

    pub(crate) fn dispatch<R: Into<DispatchRequest>>(&mut self, req: R, ctx: InitContext) {
        let (job, token) = ConcurrentJob::new();
        ctx.add_job(job);

        if let Err(e) = self.sender.send((req.into(), ctx, token)) {
            log::error!("Failed to dispatch request: {:?}", e);
        }
    }
}

/// Non-blocking request logic designed to be packed into a `DispatchRequest`
/// and handled on the `WORK_POOL` via a `Dispatcher`
pub trait RequestAction: LspRequest {
    /// Serializable response type
    type Response: ServerResponse + Send;

    /// Maximum duration this request should finish within; also see
    /// `fallback_response()`
    fn timeout() -> Duration {
        DEFAULT_REQUEST_TIMEOUT
    }

    /// Returns a response used in timeout scenarios
    fn fallback_response() -> Result<Self::Response, ResponseError>;

    /// Request processing logic
    fn handle(ctx: InitContext, params: Self::Params) -> Result<Self::Response, ResponseError>;
}