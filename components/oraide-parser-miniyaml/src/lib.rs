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

mod parser;
pub use parser::{
    Token,
    TokenKind,
    Tokenizer,
    TokenCollectionExts,
    Node,
    Nodeizer,
    IndentLevelDelta,
    Arena,
    ArenaNodeId,
    Tree,
    Treeizer,
};

mod computation;
pub use computation::{
    files_ctx::{
        FilesCtx,
        FilesCtxExt,
        FilesCtxStorage,
        TextFilesCtx,
        TextFilesCtxExt,
        TextFilesCtxStorage,
    },
    parser_ctx::{
        ParserCtx,
        ParserCtxStorage,
    },
};