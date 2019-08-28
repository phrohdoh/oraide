use std::{
    io,
    path::{
        PathBuf,
    },
};

use derive_more::{
    Display,
    From,
};

use url::Url;

use oraide_parser_miniyaml::{
    ParserCtx,
};

use oraide_span::{
    ByteIndex,
    FileId,
};

use oraide_actor::{
    Position,
};

/// A newtype over [`String`] used for [`LangServerCtx::hover`]
///
/// [`String`]: https://doc.rust-lang.org/nightly/alloc/string/struct.String.html
/// [`LangServerCtx::hover`]: trait.LangServerCtx.html#method.hover
#[derive(Debug, Clone, Display, From, PartialEq, Eq, Hash)]
pub struct Markdown(pub String);