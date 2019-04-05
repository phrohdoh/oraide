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
    types::{
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
            fn handle<O: Output>(self, out: &O) {
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
                                $req_type::handle(params)
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
    sender: mpsc::Sender<(DispatchRequest, JobToken)>,
}

impl Dispatcher {
    /// Create a new `Dispatcher` starting a new thread and channel
    pub(crate) fn new<O: Output>(out: O) -> Self {
        let (sender, rx) = mpsc::channel::<(DispatchRequest, JobToken)>();

        thread::Builder::new()
            .name("dispatch-worker".into())
            .spawn(move || {
                while let Ok((req, tok)) = rx.recv() {
                    req.handle(&out);
                    drop(tok);
                }
            })
            .unwrap();

        Self { sender }
    }

    pub(crate) fn dispatch<R: Into<DispatchRequest>>(&mut self, req: R) {
        let (_job, tok) = ConcurrentJob::new();
        // TODO: `ctx.add_job(job)`
        // https://github.com/rust-lang/rls/blob/4834d4fa7afcc265e1f5e09b7c9d59178c6b230a/rls/src/server/dispatch.rs#L138

        if let Err(e) = self.sender.send((req.into(), tok)) {
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
    fn handle(params: Self::Params) -> Result<Self::Response, ResponseError>;
}