//! Contains LSP request implementations

use lsp_types::request::{
    HoverRequest,
};

use crate::{
    dispatch::RequestAction,
    types::{
        ResponseError,
    },
};

impl RequestAction for HoverRequest {
    type Response = lsp_types::Hover;

    fn fallback_response() -> Result<Self::Response, ResponseError> {
        Ok(lsp_types::Hover {
            contents: lsp_types::HoverContents::Array(vec![]),
            range: None,
        })
    }

    fn handle(params: Self::Params) -> Result<Self::Response, ResponseError> {
        log::trace!("Got hover request in `{}` at `{}:{}`", params.text_document.uri, params.position.line, params.position.character);
        Ok(Self::Response {
            contents: lsp_types::HoverContents::Scalar(
                lsp_types::MarkedString::String("test!!!".to_string())
            ),
            range: None,
        })
    }
}