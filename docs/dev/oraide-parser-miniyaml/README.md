# `oraide-parser-miniyaml`

This package implements facilities to convert a textual MiniYaml document into a tree of nodes.

<small>In the future this may change to produce a forest (numerous trees) for a text document.</small>

Parsing is implemented in 3 _phases_:

- [Tokenization](./tokenization.md)
- [Nodeization](./nodeization.md)
- [Treeization](./treeization.md)

<small>"Tokenization" is a "real" word but the others are made up with the intent of conveying what they do while being similar to "tokenization".</small>

NOTE: These 3 phases are not the entirety of the parsing journey.

After reading the previously-linked documents you should read the [incremental recomputation](./incremental-recomputation.md) document to learn how the other packages in this project query the parsed data.