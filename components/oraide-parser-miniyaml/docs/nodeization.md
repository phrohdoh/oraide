# Parsing: The Nodeization Phase

What is "nodeization"?

The process of converting a collection of `Token`s into a collection of `Node`s.

In MiniYaml a `Node` is a line of text, essentially, so we're taking the previously-created tokens and applying structure (denoting indentation, key vs value vs comment, etc.).

---

The entrypoint to nodeization is the aptly named `Nodeizer` type which is constructed like so:

```rust
use oraide_parser_miniyaml::Nodeizer;

let tokens: Vec<Token> = unimplemented!(); // see the `tokenization` document

let mut nodeizer = Nodeizer::new(tokens.into_iter());
```

We iterate over the given tokens and determine which section of a `Node` they belong to (for example, once we find a `#` symbol all the remaining tokens are part of the comment section).

See the `next` function in `components/oraide-parser-miniyaml/src/parser/nodeizer/mod.rs`.

```rust
use oraide_parser_miniyaml::Node;

let nodes: Vec<Node> = nodeizer.run();
```

Each `Node` has 5 potential sections:

- indentation
- key
- key terminator
- value
- comment

NOTE: Not all key-having nodes will have a key terminator, but if a key terminator is present a key must also be present.

---

Go back [to the readme](../README.md).

Go back [to the tokenization](./tokenization.md) phase.

Or read about the [treeization](./treeization.md) phase.