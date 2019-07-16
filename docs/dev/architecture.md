# _OpenRA IDE_ - Architecture

The _OpenRA IDE_ project is made up of multiple codebases, each potentially containing multiple packages, that all come together to give you a better modding experience.

This particular codebase contains the "core" pieces of the _OpenRA IDE_ project:

- [`oraide-span`]: a library that defines types used for text &amp; file tracking
- [`oraide-actor`]: a primitive actor system that allows concurrent computation
- [`oraide-query-system`]: on-demand, incremental computation
- [`oraide-parser-miniyaml`]: lexing, parsing, and tree-building of MiniYaml documents
- [`oraide-sdk`]: management of [SDK]-based projects
- [`oraide-language-server`]: an LSP server for use with LSP clients (such as the Visual Studio Code extension [`oraide-vscode`])

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
- Symbol tables
    - What / Why / How
        - `ParserCtx::all_definitions`

[`oraide-span`]: ../../components/oraide-span/README.md
[`oraide-actor`]: ../../components/oraide-actor/README.md
[`oraide-parser-miniyaml`]: ../../components/oraide-parser-miniyaml/README.md
[`oraide-query-system`]: ../../components/oraide-query-system/README.md
[`oraide-sdk`]: ../../components/oraide-sdk/README.md
[`oraide-language-server`]: ../../components/oraide-language-server/README.md
[`oraide-vscode`]: ../../editors/vscode/README.md
[SDK]: https://github.com/OpenRA/OpenRAModSDK/