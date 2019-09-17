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
        ops::Deref,
    }
};

mod sdk_ctx;
pub use sdk_ctx::{
    SdkCtx,
    SdkCtxStorage,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameId(String);

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
    id: GameId,
}

impl Game {
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Game,
        GameId,
    };

    #[test]
    fn game_id_fn_returns_id() {
        // Arrange
        let game = Game {
            id: GameId("foo".into()),
        };

        // Act
        let id = game.id();

        // Assert
        assert_eq!("foo", id);
    }
}
