// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    collections::{
        HashMap,
    },
    fs::{
        self,
        DirEntry,
    },
    path::{
        PathBuf,
        Path,
        Component,
    }
};

use slog::o;

pub struct Project {
    pub root_dir_path: PathBuf,
    pub games: HashMap<String, Game>,
}

impl Project {
    // This ctor wraps `_new_from_abs_dir`, which is the actual implementation, in logging
    /// Create a `Project` instance by analyzing the directory structure of `project_root_dir`
    pub fn new_from_abs_dir<P: AsRef<Path>>(abs_dir_path: P) -> Result<Self, String> {
        slog_scope::scope(
            &slog_scope::logger().new(o!(
                "project-root-dir" => format!("{}", abs_dir_path.as_ref().display()),
            )),
            || Self::_new_from_abs_dir(abs_dir_path)
        )
    }

    fn _new_from_abs_dir<P: AsRef<Path>>(abs_dir_path: P) -> Result<Self, String> {
        let project_root_dir: &Path = abs_dir_path.as_ref();
        log::trace!("Attempting to create a `Project` from path: `{}", project_root_dir.display());

        let metadata = project_root_dir
            .metadata()
            .map_err(|e| format!("Failed to read metadata of `{}`: {}", project_root_dir.display(), e))?;

        if !metadata.is_dir() {
            return Err(format!(
                "Given path expected to be a directory, but was not: {}",
                project_root_dir.display()
            ));
        }

        let rel_mods_dir = project_root_dir.join("mods/");
        let read_dir = fs::read_dir(&rel_mods_dir)
            .map_err(|e| format!("Failed to read `{}`: {}", rel_mods_dir.display(), e))?;

        let games = read_dir.filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            let game = match Game::new_from_dir_entry(&entry) {
                Ok(game) => game,
                Err(e_str) => {
                    log::debug!(
                        "Skipping `{}`: {}",
                        path.display(),
                        e_str
                    );

                    return None;
                },
            };

            let tup = (game.id.clone(), game);
            log::trace!("Adding game `{}` at `{}` to gamedb", tup.0, path.display());
            Some(tup)
        }).collect::<HashMap<_, _>>();

        Ok(Self {
            root_dir_path: project_root_dir.to_path_buf(),
            games,
        })
    }
}

macro_rules! pathbuf_from_components {
    ($base:expr, $($seg:expr),+) => {{
        let mut base: std::path::PathBuf = $base.into();
        $(
            base.push($seg);
        )*
        base
    }};
}

#[derive(Clone, Debug)]
pub struct Game {
    id: String,
}

impl Game {
    pub(crate) fn new_from_abs_dir(abs_dir_path: PathBuf) -> Result<Self, String> {
        slog_scope::scope(&slog_scope::logger(), || Self::_new_from_abs_dir(abs_dir_path))
    }

    fn _new_from_abs_dir(abs_dir_path: PathBuf) -> Result<Self, String> {
        log::trace!("Attempting to create a `Game` from an absolute path: `{}`", abs_dir_path.display());

        match abs_dir_path.components().last() {
            Some(Component::Normal(s)) => match s.to_str() {
                Some(s) => Ok(Self { id: s.to_owned() }),
                _ => Err(format!("Failed to extract ID component from `{}`", abs_dir_path.display())),
            },
            _ => Err(format!("Last component of `{}` is not a file or directory", abs_dir_path.display())),
        }
    }

    pub(crate) fn new_from_dir_entry(entry: &DirEntry) -> Result<Self, String> {
        slog_scope::scope(&slog_scope::logger(), || Self::_new_from_dir_entry(entry))
    }

    fn _new_from_dir_entry(entry: &DirEntry) -> Result<Self, String> {
        let path = entry.path();
        log::trace!("Attempting to create a `Game` from a directory entry: `{}`", path.display());

        let metadata = fs::metadata(&path)
            .map_err(|e| format!(
                "Failed to read metadata of `{}`: {}",
                path.display(),
                e,
            ))?;

        if !metadata.is_dir() {
            return Err(format!(
                "Given entry expected to be a directory, but was not: `{}`",
                path.display(),
            ));
        }

        let game = match Game::new_from_abs_dir(path) {
            Ok(g) => g,
            Err(e_str) => return Err(format!(
                "Failed to create `Game` instance because: {}",
                e_str,
            )),
        };

        Ok(game)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    /// Absolute path to this `Game`'s directory
    pub fn abs_path(&self, project: &Project) -> PathBuf {
        pathbuf_from_components!(
            &project.root_dir_path,
            "mods",
            &self.id
        )
    }

    /// Absolute path to this `Game`'s manifest (`mod.yaml`)
    pub fn manifest_path_abs(&self, project: &Project) -> PathBuf {
        pathbuf_from_components!(
            self.abs_path(project),
            "mod.yaml"
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
