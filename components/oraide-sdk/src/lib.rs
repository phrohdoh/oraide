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

use {
    std::{
        fmt,
        ops::Deref,
        path::PathBuf,
    }
};

mod sdk_ctx;
pub use sdk_ctx::{
    SdkCtx,
    SdkCtxStorage,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameId(String);

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Deref for GameId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<String> for GameId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A "mod" in OpenRA terms.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Game {
    pub id: GameId,

    /// The workspace root-relative path to this `Game`'s root directory.
    ///
    /// # Example
    ///
    /// Assume the following file structure.
    ///
    /// ```
    /// <workspace root>
    /// ├── artsrc
    /// ├── engine
    /// │  └── mods
    /// │     ├── all
    /// │     │  └── mod.yaml
    /// │     ├── cnc
    /// │     │  └── mod.yaml
    /// │     └── ra
    /// │        └── mod.yaml
    /// ├── LICENSE
    /// ├── mod.config
    /// ├── mods
    /// │  ├── bar
    /// │  │  ├── mod.yaml
    /// │  │  └── rules
    /// │  │     └── civilian-structures.yaml
    /// │  └── foo
    /// │     └── mod.yaml
    /// └── packaging
    /// ```
    ///
    /// For the `cnc` game this would be `engine/mods/cnc/`.
    /// For the `bar` game this would be `mods/bar/`.
    /// For the `foo` game this would be `mods/foo/`.
    pub rel_root_path: PathBuf,
}

impl Game {
    pub fn new(
        id: GameId,
        rel_root_path: PathBuf,
    ) -> Self {
        Self {
            id,
            rel_root_path,
        }
    }
}