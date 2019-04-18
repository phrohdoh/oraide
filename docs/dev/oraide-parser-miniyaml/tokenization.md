# Parsing: The Tokenization Phase

What is tokenization?

From [Wikipedia](https://en.wikipedia.org/wiki/Lexical_analysis#Tokenization):

> Tokenization is the process of demarcating and possibly classifying sections of a string of input characters. The resulting tokens are then passed on to some other form of processing. The process can be considered a sub-task of parsing input.

---

Tokenization is a straight-forward process (even though the MiniYaml format is not simple).

The entrypoint to tokenization is the aptly named `Tokenizer` type which is constructed like so:

```rust
use oraide_span::FileId;
use oraide_parser_miniyaml::Tokenizer;

let source_text: String = unimplemented!(); // get this from somewhere (most likely a file on-disk)
let file_id: FileId = unimplemented!(); // see the `incremental recomputation` document

let mut tokenizer = Tokenizer::new(file_id, &source_text);
```

Given the "raw" input of a text document we split the text into `Token`s ("words" and symbols), basically.

See the `consume_token` function in `components/oraide-parser-miniyaml/src/parser/tokenizer/mod.rs`.

```rust
use oraide_parser_miniyaml::Token;

let tokens: Vec<Token> = tokenizer.run();
```

Each `Token` has a `kind` (identifier, `@` symbol, `~` symbol, whitespace, etc.) and a `span`, which is a pair of byte indicies pointing into the source file.

The raw text is not stored in the `Token` type but can be queried, which will be covered in the [incremental recomputation](./incremental-recomputation.md) document.