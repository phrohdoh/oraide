// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! The `server` module contains the types and implementations that make up our
//! LSP implementation.

use std::{
    fmt,
    io::{
        self,
        BufRead,
        Write as _,
    },
    sync::{
        Arc,
        atomic::{
            AtomicU64,
            Ordering,
        },
    },
};

use lsp_types::{
    request::{
        self as lsp_request,
        Request as LspRequest,
    },
    notification::{
        self as lsp_notification,
        Notification as LspNotification,
    }
};

use jsonrpc_core::{
    self as jsonrpc,
    Id as JsonId,
    response,
    version,
    ErrorCode,
};

use crate::{
    lsp::{
        Request,
        Notification,
        RequestId,
        RawMessage,
    },
    dispatch::{
        Dispatcher,
    },
    context::{
        Context,
        InitContext,
    },
};

pub(crate) const NOT_INITIALIZED_CODE: ErrorCode = ErrorCode::ServerError(-32002);

/// Indicates how the server should proceed
#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum ServerStateChange {
    Continue,
    Exit { code: i32 },
}

pub struct LangServerService<O: Output> {
    reader: Box<dyn MsgReader + Send + Sync>,
    output: O,
    ctx: Context,
    dispatcher: Dispatcher,
}

impl<O: Output> LangServerService<O> {
    /// Construct a new `LangServerService` instance
    pub fn new(
        reader: Box<dyn MsgReader + Send + Sync>,
        output: O,
    ) -> LangServerService<O> {
        let dispatcher = Dispatcher::new(output.clone());

        Self {
            reader,
            output,
            ctx: Context::new(),
            dispatcher,
        }
    }

    pub fn run(mut self) -> i32 {
        loop {
            match self.handle_message() {
                ServerStateChange::Continue => (),
                ServerStateChange::Exit { code } => return code,
            }
        }
    }

    /// Read a message from the input and handle it with the
    /// appropriate action
    ///
    /// # Returns
    /// A `ServerStateChange` that describes how the service should
    /// proceed now that the message has been handled
    pub fn handle_message(&mut self) -> ServerStateChange {
        let msg = match self.reader.read_message() {
            Some(m) => m,
            _ => {
                log::error!("Failed to read message");
                self.output.send_failure(JsonId::Null, jsonrpc::Error::parse_error());
                return ServerStateChange::Exit { code: 101 };
            },
        };

        log::trace!("Handling message `{}`", msg);

        let raw_msg = match RawMessage::try_from_str(&msg) {
            Ok(Some(rm)) => rm,
            Ok(None) => return ServerStateChange::Continue,
            Err(e) => {
                log::error!("Failed to parse into raw message: {:?}", e);
                self.output.send_failure(JsonId::Null, jsonrpc::Error::parse_error());
                return ServerStateChange::Exit { code: 101 };
            },
        };

        log::debug!("Parsed message `{:?}`", raw_msg);

        if let Err(e) = self.dispatch_message(&raw_msg) {
            log::error!("Dispatch error: {:?}, message: `{}`", e, msg);
            self.output.send_failure(raw_msg.id, e);
            return ServerStateChange::Exit { code: 101 };
        }

        ServerStateChange::Continue
    }

    fn wait_for_concurrent_jobs(&mut self) {
        match &self.ctx {
            Context::Init(ctx) => ctx.wait_for_concurrent_jobs(),
            _ => {},
        }
    }

    fn dispatch_message(&mut self, msg: &RawMessage) -> Result<(), jsonrpc::Error> {
        macro_rules! action {
            (
                $method: expr;
                notifications: $($notif: ty),*;
                blocking_requests: $($blocking_request:ty),*;
                requests: $($request: ty),*;
            ) => {
                match $method.as_str() {
                    $(
                        <$notif as LspNotification>::METHOD => {
                            let notif: Notification<$notif> = msg.parse_as_notification()?;
                            if let Ok(mut ctx) = self.ctx.inited() {
                                if notif.dispatch(&mut ctx, self.output.clone()).is_err() {
                                    log::debug!("Error handling notification: {:?}", msg);
                                }
                            } else {
                                log::warn!("Server has not yet received an `initialize` request, cannot handle `{}`", $method);
                                // NOTE: We are deliberately _not_ sending a
                                // failure message because notifications
                                // are not requests, meaning notifications
                                // should not be responded to, they are
                                // one-way messages.
                            }
                        },
                    )*

                    $(
                        <$blocking_request as LspRequest>::METHOD => {
                            let req: Request<$blocking_request> = msg.parse_as_request()?;

                            // Block until all non-blocking requests have been
                            // handled ensuring ordering.
                            self.wait_for_concurrent_jobs();

                            let req_id = req.id.clone();

                            match req.blocking_dispatch(&mut self.ctx, &self.output) {
                                Ok(resp) => resp.send(req_id, &self.output),
                                Err(ResponseError::Empty) => {
                                    log::error!("Error handling `{}`", $method);
                                    self.output.send_failure_message(
                                        req_id,
                                        ErrorCode::InternalError,
                                        "An unknown error occurred",
                                    );
                                },
                                Err(ResponseError::Message(code, msg)) => {
                                    log::error!("Error handling `{}`: {}", $method, msg);
                                    self.output.send_failure_message(req_id, code, msg);
                                },
                            }
                        },
                    )*

                    $(
                        <$request as LspRequest>::METHOD => {
                            let req: Request<$request> = msg.parse_as_request()?;

                            if let Ok(ctx) = self.ctx.inited() {
                                self.dispatcher.dispatch(req, ctx);
                            } else {
                                log::warn!("Server has not yet received an `initialize` request, cannot handle `{}`", $method);
                                self.output.send_failure_message(req.id, NOT_INITIALIZED_CODE, "not yet received `initialize` request".to_owned());
                            }
                        },
                    )*

                    _ => log::debug!("method `{}` not handled", $method),
                }
            }
        }

        action!(
            msg.method;
            notifications:
                lsp_notification::Initialized;
            blocking_requests:
                lsp_request::Initialize;
            requests:
                lsp_request::HoverRequest;
        );

        Ok(())
    }
}

/// Anything that can read LSP server input messages
pub trait MsgReader {
    /// Read the next input message
    fn read_message(&self) -> Option<String>;
}

/// A message reader that gets input from `stdin`
pub struct StdinMsgReader;

impl MsgReader for StdinMsgReader {
    fn read_message(&self) -> Option<String> {
        let stdin = io::stdin();
        let mut locked = stdin.lock();

        match read_message(&mut locked) {
            Ok(message) => Some(message),
            Err(err) => {
                log::error!("Error reading message: {:?}", err);
                None
            }
        }
    }
}

fn read_message(input: &mut impl BufRead) -> Result<String, io::Error> {
    log::warn!("Reading message...");
    let mut content_length = None;

    // Read the header section
    loop {
        let mut buf = String::new();
        input.read_line(&mut buf)?;

        if buf.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Unexpected EOF encountered in the middle of LSP headers",
            ));
        }

        // An empty ("\r\n"-only) line marks the end of the header section
        if buf == "\r\n" {
            break;
        }

        let res = buf.split(':').collect::<Vec<_>>();
        if res.len() != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Header `{}` is malformed", buf),
            ));
        }

        let hdr_name = res[0].trim().to_lowercase();
        let hdr_value = res[1].trim();
        log::warn!("Header `{}` = `{}`", hdr_name, hdr_value);

        match hdr_name.as_ref() {
            "content-length" => {
                content_length = Some(
                    usize::from_str_radix(hdr_value, 10)
                        .map_err(|_e| io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "Could not parse header `{}`'s value `{}` as unsigned integer",
                                hdr_name,
                                hdr_value,
                            ),
                        )
                    )?
                );
            },
            "content-type" => {
                // Previous versions of the LSP used 'utf8' which isn't a
                // valid encoding constant but we support 'utf8' for backwards
                // compatibility.
                if hdr_value != "utf8" && hdr_value != "utf-8" {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Content type `{}` is invalid (only `utf-8` is supported)", hdr_value),
                    ));
                }
            },
            _ => {
                // The LSP spec does not say what to do if an invalid/unknown header is found
                log::debug!("Ignoring unknown header `{}`", hdr_name);
            },
        }
    }

    let content_length = match content_length {
        Some(l) => l,
        _ => return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Message is missing `content-length` header",
        )),
    };

    log::trace!("Reading {} bytes", content_length);

    let mut content = vec![0; content_length];
    input.read_exact(&mut content)?;

    String::from_utf8(content)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Anything that can send notifications and responses to an LSP client
pub trait Output: Sync + Send + Clone + 'static {
    /// Send a response string along this output
    fn send_response(&self, output: String);

    /// Gets a new unique ID
    fn gen_unique_id(&self) -> RequestId;

    /// Sends a successful notification or response along this output
    fn send_success<D: serde::Serialize + fmt::Debug>(&self, id: RequestId, data: &D) {
        let data = match serde_json::to_string(data) {
            Ok(d) => d,
            Err(e) => {
                log::error!("Could not serialize data for success message `{:?}`: {}", data, e);

                return;
            },
        };

        let output = format!("{{\"jsonrpc\":\"2.0\",\"id\":{},\"result\":{}}}", id, data);
        self.send_response(output);
    }

    /// Notify the client of a failure
    fn send_failure(&self, id: JsonId, error: jsonrpc::Error) {
        let resp = response::Failure {
            id,
            error,
            jsonrpc: Some(version::Version::V2),
        };

        self.send_response(serde_json::to_string(&resp).unwrap());
    }

    /// Notify the client of a failure with the given diagnostic message
    fn send_failure_message(&self, id: RequestId, code: jsonrpc::ErrorCode, msg: impl Into<String>) {
        let error = jsonrpc::Error {
            code,
            message: msg.into(),
            data: None,
        };

        self.send_failure(JsonId::from(&id), error);
    }
}

/// An output that sends notifications and responses over `stdout`
#[derive(Clone)]
pub struct StdoutOutput {
    next_id: Arc<AtomicU64>,
}

impl StdoutOutput {
    /// Construct a new `StdoutOutput`
    pub fn new() -> Self {
        Self {
            next_id: Arc::new(AtomicU64::new(1))
        }
    }
}

/// A response that just acknowledges receipt of its request.
#[derive(Debug, serde::Serialize)]
pub struct Ack;

/// A response to some request
pub trait Response {
    /// Send the reponse along the given output
    fn send<O: Output>(self, id: RequestId, output: &O);
}

impl<R: serde::Serialize + fmt::Debug> Response for R {
    fn send<O: Output>(self, id: RequestId, out: &O) {
        out.send_success(id, &self)
    }
}

/// The lack of a response to a request;
#[derive(Debug)]
pub struct NoResponse;

impl Response for NoResponse {
    fn send<O: Output>(self, _id: RequestId, _out: &O) {}
}

/// Wrapper for a response error
#[derive(Debug)]
pub enum ResponseError {
    /// Error with no special response to the client
    Empty,

    /// Error with a response to the client
    Message(jsonrpc::ErrorCode, String),
}

/// A request that blocks the input whilst being handled
pub trait BlockingRequestAction: LspRequest {
    type Response: Response + fmt::Debug;

    fn handle<O: Output>(
        id: RequestId,
        params: Self::Params,
        // We use `Context`, instead of `InitContext`, here because of the init request
        ctx: &mut Context,
        output: O,
    ) -> Result<Self::Response, ResponseError>;
}

impl<A: BlockingRequestAction> Request<A> {
    pub fn blocking_dispatch<O: Output>(
        self,
        // We use `Context`, instead of `InitContext`, here because of the init request
        ctx: &mut Context,
        output: &O,
    ) -> Result<A::Response, ResponseError> {
        A::handle(self.id, self.params, ctx, output.clone())
    }
}

/// An action taken in response to some notification from the client.
/// Blocks the input channel whilst being handled.
pub trait BlockingNotificationAction: LspNotification {
    /// Handles this notification
    fn handle<O: Output>(_: Self::Params, _: &mut InitContext, _: O) -> Result<(), ()>;
}

impl<A: BlockingNotificationAction> Notification<A> {
    pub fn dispatch<O: Output>(self, ctx: &mut InitContext, out: O) -> Result<(), ()> {
        A::handle(self.params, ctx, out)?;
        Ok(())
    }
}

impl Output for StdoutOutput {
    fn send_response(&self, output: String) {
        let o = format!("Content-Length: {}\r\n\r\n{}", output.len(), output);
        log::trace!("Sending response: {:?}", o);

        let stdout = io::stdout();
        let mut lock = stdout.lock();
        write!(lock, "{}", o).unwrap();
        lock.flush().unwrap();
    }

    fn gen_unique_id(&self) -> RequestId {
        RequestId::Num(self.next_id.fetch_add(1, Ordering::SeqCst))
    }
}

pub fn capabilities(_ctx: &Context) -> lsp_types::ServerCapabilities {
    lsp_types::ServerCapabilities {
        hover_provider: true.into(),
        ..Default::default()
    }
}