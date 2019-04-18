# Repository Structure

At the top-level this repository is a [Rust] binary application (because `src/main.rs` exists).

## Components

This project has multiple components which you will find in the aptly named `components` directory.

Each "component" is a [Rust] library with its own set of dependencies, features, documentation, etc.

## Editor Support

Editor support is implemented in projects under the `editors` directory.

There is a [Visual Studio Code] reference implementation in the `editors/vscode` directory.

[Rust]: https://www.rust-lang.org/
[Visual Studio Code]: https://code.visualstudio.com/