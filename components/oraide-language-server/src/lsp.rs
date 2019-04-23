// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// The `lsp` module provides foundational types
/// for an LSP implementation.

use std::{
    marker::PhantomData,
    time::Instant,
    fmt,
};

use serde::de::Deserialize as _;

use lsp_types::{
    request::Request as LspRequest,
    notification::Notification as LspNotification,
};

use jsonrpc_core::{
    self as jsonrpc,
    Id as JsonId,
};

/// A request ID as defined by the LSP spec
#[derive(Debug, PartialEq, Clone, Hash, Eq)]
pub enum RequestId {
    Str(String),
    Num(u64),
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestId::Str(ref s) => write!(f, "\"{}\"", s),
            RequestId::Num(n) => write!(f, "{}", n),
        }
    }
}

impl<'a> From<&'a RequestId> for JsonId {
    fn from(request_id: &RequestId) -> Self {
        match request_id {
            RequestId::Str(ref s) => JsonId::Str(s.to_string()),
            RequestId::Num(n) => JsonId::Num(*n),
        }
    }
}

/// A request that gets sent to the server as JSON
pub struct Request<A: LspRequest> {
    /// The unique request ID
    pub id: RequestId,

    /// The time the request was received / processed by the `MsgReader`
    pub received: Instant,

    /// The extra, action-specific, parameters
    pub params: A::Params,

    /// This `Request`'s handler action
    pub _action: PhantomData<A>,
}

impl<A: LspRequest> Request<A> {
    /// Create a `Request` instance
    pub fn new(id: RequestId, params: A::Params) -> Request<A> {
        Self {
            id,
            params,
            received: Instant::now(),
            _action: PhantomData,
        }
    }
}

/// A notification that gets sent to the server as JSON
#[derive(Debug, PartialEq)]
pub struct Notification<A: LspNotification> {
    /// The extra, action-specific, parameters
    pub params: A::Params,

    /// The action responsible for this notification
    pub _action: PhantomData<A>,
}

impl<A: LspNotification> Notification<A> {
    /// Create a new `Notification` with the given `params`
    pub fn new(params: A::Params) -> Notification<A> {
        Self { params, _action: PhantomData }
    }
}

#[derive(Debug, PartialEq)]
pub struct RawMessage {
    pub method: String,
    pub id: JsonId,
    pub params: serde_json::Value,
}

impl RawMessage {
    pub fn parse_as_request<'de, R>(&'de self) -> Result<Request<R>, jsonrpc::Error>
        where R: LspRequest,
              <R as LspRequest>::Params: serde::Deserialize<'de>
    {
        let id = match self.id {
            JsonId::Num(n) => Some(RequestId::Num(n)),
            JsonId::Str(ref s) => Some(RequestId::Str(s.to_owned())),
            _ => None,
        };

        let params = R::Params::deserialize(&self.params)
            .map_err(|e| {
                log::debug!("Error parsing as request: {}", e);
                jsonrpc::Error::invalid_params(format!("{}", e))
            })?;

        match id {
            Some(id) => Ok(Request {
                id,
                params,
                received: Instant::now(),
                _action: PhantomData,
            }),
            _ => Err(jsonrpc::Error::invalid_request()),
        }
    }

    pub fn parse_as_notification<'de, T>(&'de self) -> Result<Notification<T>, jsonrpc::Error>
        where T: LspNotification,
              <T as LspNotification>::Params: serde::Deserialize<'de>
    {
        let params = T::Params::deserialize(&self.params)
            .map_err(|e| {
                log::debug!("Error parsing as notification: {}", e);
                jsonrpc::Error::invalid_params(format!("{}", e))
            })?;

        Ok(Notification { params, _action: PhantomData })
    }

    pub fn try_from_str(msg: &str) -> Result<Option<Self>, jsonrpc::Error> {
        let cmd: serde_json::Value = serde_json::from_str(msg)
            .map_err(|_e| jsonrpc::Error::parse_error())?;

        // Requests must have an ID while notifications must not
        let id = cmd.get("id")
            .map_or(JsonId::Null, |id| serde_json::from_value(id.to_owned()).unwrap());

        let method = match cmd.get("method") {
            Some(m) => m,
            _ => {
                // This is a response to one of our requests
                return Ok(None);
            },
        }.as_str().ok_or_else(jsonrpc::Error::invalid_request)?.to_owned();

        let params = match cmd.get("params").map(ToOwned::to_owned) {
            Some(params @ serde_json::Value::Object(..)) | Some(params @ serde_json::Value::Array(..)) => params,
            Some(serde_json::Value::Null) | None => serde_json::Value::Null,
            _ => return Err(jsonrpc::Error::invalid_request())
        };

        Ok(Some(Self {
            method,
            id,
            params,
        }))
    }
}