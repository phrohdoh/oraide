# `oraide-miniyaml`

A library, written in [Rust](https://www.rust-lang.org/), which implements lexing, parsing, and tree-building for OpenRA's MiniYaml
format based on [brendanzab's mltt-parse crate](https://github.com/brendanzab/rust-nbe-for-mltt/crates/mltt-parse/src/lexer.rs).

This project contains a simple executable target (`src/main.rs`) that you can easily run like so:

```
$ cargo run -- /path/to/some/miniyaml/file.yaml
```