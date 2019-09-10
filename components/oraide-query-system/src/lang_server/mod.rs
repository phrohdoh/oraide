// This file is part of oraide.  See <https://github.com/Phrohdoh/oraide>.
// 
// oraide is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License version 3
// as published by the Free Software Foundation.
// 
// oraide is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with oraide.  If not, see <https://www.gnu.org/licenses/>.

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