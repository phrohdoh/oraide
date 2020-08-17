# `oraide`

## project plan

A suite of tools to aide in the development of [OpenRA]-based games.

## actual project status

Allows you to [lex] [MiniYaml] into componentized, spanned lines for the
purpose of [linting], or whatever else you can think of.

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

## running the code

For the time being, this project is primarily a library (not an executable)
so there is not much to run, but there is a binary target (`src/main.rs`) that
is used to manually test the library's functionality.

From the root of this repository (the directory containing the file you're
currently reading), execute the following in your shell:

```
cargo run test-miniyaml-files/simple.yaml
```

You should see output similar to the following:

<details><summary>command output</summary>

```
raw     = "E2:\n"
indent  = None
key     = Some("E2")
key_sep = Some(":")
value   = None
comment = None
term    = Some("\n")

raw     = "    Inherits: ^Soldier\n"
indent  = Some("    ")
key     = Some("Inherits")
key_sep = Some(":")
value   = Some("^Soldier")
comment = None
term    = Some("\n")

raw     = "    Inherits@experience: ^GainsExperience\n"
indent  = Some("    ")
key     = Some("Inherits@experience")
key_sep = Some(":")
value   = Some("^GainsExperience")
comment = None
term    = Some("\n")

raw     = "\tValued:\n"
indent  = Some("\t")
key     = Some("Valued")
key_sep = Some(":")
value   = None
comment = None
term    = Some("\n")

raw     = "\t\t# Cost: 200\n"
indent  = Some("\t\t")
key     = None
key_sep = None
value   = None
comment = Some("# Cost: 200")
term    = Some("\n")

raw     = "  \tCost: 200\n"
indent  = Some("  \t")
key     = Some("Cost")
key_sep = Some(":")
value   = Some("200")
comment = None
term    = Some("\n")

raw     = "\t  non_ascii: 请务必取代#idk\n"
indent  = Some("\t  ")
key     = Some("non_ascii")
key_sep = Some(":")
value   = Some("请务必取代")
comment = Some("#idk")
term    = Some("\n")

raw     = "\t  请务必取代: non_ascii_key #idk\n"
indent  = Some("\t  ")
key     = Some("请务必取代")
key_sep = Some(":")
value   = Some("non_ascii_key ")
comment = Some("#idk")
term    = Some("\n")
```

</details>


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
