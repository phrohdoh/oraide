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

### Parser Phases

The parser is implemented in 3 phases:

<details>

<summary>the tokenizer phase</summary>

The entrypoint into tokenization is the `Tokenizer` type.

To create a `Tokenizer` you need 2 things:

- the underlying text to be tokenized (split into `Token`s)
- a `FileId` (which uniquely identifies a source file, we'll get to this later)

```rust
use oraide_span::FileId;
use oraide_parser_miniyaml::Tokenizer;

let source_text: String = unimplemented!(); // get this from somewhere (probably a file on-disk)
let file_id: FileId = unimplemented!(); // again, we'll get to this later
let mut tokenizer = Tokenizer::new(file_id, &source_text);
```

Performing tokenization is simple.

```rust
let tokens: Vec<Token> = tokenizer.run();
```

</details>

<details>

<summary>the nodeizer phase</summary>

The entrypoint into nodeization is the `Nodeizer` type.

To create a `Nodeizer` you need 1 thing:

- an `Iterator` of `Token`s to be nodeized (grouped into `Node`s)

```rust
use oraide_parser_miniyaml::Nodeizer;

let mut nodeizer = Nodeizer::new(tokens.into_iter()); // `tokens` is from the previous code snippet
```

Performing nodeization is simple.

```rust
let nodes = nodeizer.run();
```
</details>

<details>

<summary>the treeizer phase</summary>

The entrypoint into treeization is the `Treeizer` type.

To create a `Treeizer` you need 1 thing:

- an `Iterator` of `Node`s to be treeized (massaged into a tree-like structure with relationships [parent, child, sibling, etc.])

```rust
use oraide_parser_miniyaml::Treeizer;

let mut treeizer = Treeizer::new(nodes.into_iter()); // `nodes` is from the previous code snippet
```

Performing treeization is simple.

```rust
use oraide_parser_miniyaml::Tree;

let tree: Tree = treeizer.run();
```

You now have a `Tree` that you can:

- determine which items are top-level (actor, weapon, etc. definitions)
- execute lint rules against (once we have lints)
- ...

</details>

But you don't really want, or need, to do any of this thanks to...

### Memoized Computations

The `oraide-parser-miniyaml` package makes it simple to turn source text into a tree!

```rust
use oraide_span::{
    FileId,
};

use oraide_parser_miniyaml::{
    Database,
    ParserCtx,
    ParserCtxExt,
    Tree,
};

let mut db = Database::default();
let file_id: FileId = db.add_file("example.yaml", "Hello:\n");
let tree: Tree = db.file_tree(file_id);
```

Hey would you look at that, a `FileId` being created, promise fulfilled!

Let's start with understanding this `Database` type.

#### The Database

This `Database` type is primarily driven by `FileId`s which are created by calling `db.add_file(name, contents)` (which returns a newly-created `FileId`).

</details>

## oraide-sdk

TODO

## oraide-language-server

TODO