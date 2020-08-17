# `oraide`

## project plan

A suite of tools to aide in the development of [OpenRA]-based games.

## actual project status

Allows you to [lex] [MiniYaml] into componentized, spanned lines for the
purpose of [linting], or whatever else you can think of.

See `src/main.rs` for an example of what is currently possible.

Since lexing is implemented, the next piece of this library to implement is
building a tree out of the lexed lines.

## technologies used

- the [Rust] programming language (see the [rust-toolchain] file for
  version info)
- Microsoft's [Visual Studio Code] (some snippets in-tree)

## compiling the code

From the root of this repository (the directory containing the file you're
currently reading), execute the following in your shell:

```
cargo build
```

## contributing

`oraide` is not yet ready for contributions.

## license ([AGPLv3])

Read [LICENSE-AGPLv3] for details.

[LICENSE-AGPLv3]: ./LICENSE-AGPLv3
[AGPLv3]: https://www.gnu.org/licenses/agpl-3.0
[OpenRA]: https://openra.net
[lex]: https://en.wikipedia.org/wiki/Lexical_analysis
[MiniYaml]: https://www.openra.net/book/glossary.html#miniyaml
[linting]: https://en.wikipedia.org/wiki/Lint_%28software%29
[Rust]: https://www.rust-lang.org/
[rust-toolchain]: ./rust-toolchain
[Visual Studio Code]: https://code.visualstudio.com/
