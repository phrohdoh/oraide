# _OpenRA IDE_ - Architecture

The _OpenRA IDE_ project is made up of multiple codebases, each potentially containing multiple packages, that all come together to give you a better modding experience.

This particular codebase contains the "core" pieces of the _OpenRA IDE_ project:

- `oraide-parser-miniyaml`: lexing, parsing, and tree-building of MiniYaml documents
- `oraide-sdk`: SDK-based project management
- `oraide-language-server`: an LSP server for use with LSP clients (such as the Visual Studio Code extension `oraide-vscode`)

Each of these builds on top of the previous ones.

## High-Level Overview

Things to cover:
- Scanning the workspace root for `mods/`
- Reading all mod manifests, which yields more files to read
- Reading all the files listed in the manifests
- What "reading a file" encompasses, in practice
    - lexing: producing a stream of tokens
    - parsing: producing a stream of nodes
    - tree-building: organizing the nodes into a data structure that looks like a tree (an `Arena`)
- Symbol tables (this doesn't exist in the code yet)
    - What / Why / How

## oraide-parser-miniyaml

This package converts a single text document into a single tree of nodes.

<small>In the future this may change to produce a single forest (numerous trees) for a single text document.</small>

<details>
<summary>MiniYaml parsing</summary>

The `oraide-parser-miniyaml` package is implemented in 2 parts:

- The underlying parser phases
- Memoized computations
