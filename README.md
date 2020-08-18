# `oraide`

## project plan

A suite of *tools* to aide in the development of [OpenRA]-based games.

> OpenRA is a general-purpose [RTS] [game engine] with *strong modding support*.

The goal of this project is not to produce [libraries] for developers, although
that will happen, but to create tools (such as command-line applications,
editor extensions, etc.) that reduce friction in game development.

## actual project status

Exposes functionality to [lex] [MiniYaml], a textual file format custom to
OpenRA, into componentized, spanned lines which can be used to implement basic
[linting], but not much else currently.

See the "[running the code](##-running-the-code)" section of this
file for more information.

---

Note that although half of the name "MiniYaml" is "Yaml", and the `.yaml` file
extension is used, it **is not [YAML]**.

Due to the fact that MiniYaml and YAML are indentation-based,
syntax-highlighting a MiniYaml file as YAML generally works well enough.

## technologies used

- the [Rust] programming language (see the [rust-toolchain] file for
  version info)
  - [Cargo] for project management / building / etc
- Microsoft's [Visual Studio Code] (some snippets in-tree)
  - use whatever works best for you

## compiling the code

From the root of this repository (the directory containing the file you're
currently reading), execute the following in your shell.

```
cargo build
```

You should see output similar to the following.

<details><summary>command output</summary>

```
Compiling oraide v0.0.0 (/your/local/path/to/oraide)
    Finished dev [unoptimized + debuginfo] target(s) in 0.81s
```
</details>

Note that by default `cargo` uses its built-in *dev* profile, not *release*.
This may have performance implications.
You can read more [here](https://doc.rust-lang.org/cargo/reference/profiles.html#dev).

## running the code

For the time being, this project is primarily a library (not an executable)
so there is not much to run, but there is a binary target (code in
`src/main.rs`) that is used to manually test the library's functionality.

From the root of this repository (the directory containing the file you're
currently reading), execute the following in your shell.

```
cargo run test-miniyaml-files/exploding-barrel.yaml
```

You should see output similar to the following.

<details><summary>command output</summary>

Notice that the lines have been split into components (`indent`, `key`, etc.).
Internally these components are byte index spans, but the text of those spans
is displayed here.

```
raw     = "exploding-barrel:\n"
indent  = None
key     = Some("exploding-barrel")
key_sep = Some(":")
value   = None
comment = None
term    = Some("\n")

raw     = "    Tooltip:\n"
indent  = Some("    ")
key     = Some("Tooltip")
key_sep = Some(":")
value   = None
comment = None
term    = Some("\n")

raw     = "        Name: barrels\n"
indent  = Some("        ")
key     = Some("Name")
key_sep = Some(":")
value   = Some("barrels")
comment = None
term    = Some("\n")

raw     = "    Health:\n"
indent  = Some("    ")
key     = Some("Health")
key_sep = Some(":")
value   = None
comment = None
term    = Some("\n")

raw     = "        HP: 5\n"
indent  = Some("        ")
key     = Some("HP")
key_sep = Some(":")
value   = Some("5")
comment = None
term    = Some("\n")

raw     = "    Explodes:\n"
indent  = Some("    ")
key     = Some("Explodes")
key_sep = Some(":")
value   = None
comment = None
term    = Some("\n")

raw     = "        Weapon: large-barrel-explode\n"
indent  = Some("        ")
key     = Some("Weapon")
key_sep = Some(":")
value   = Some("large-barrel-explode")
comment = None
term    = Some("\n")

raw     = "    MapEditorData:\n"
indent  = Some("    ")
key     = Some("MapEditorData")
key_sep = Some(":")
value   = None
comment = None
term    = Some("\n")

raw     = "        Categories: props, dangerous-props\n"
indent  = Some("        ")
key     = Some("Categories")
key_sep = Some(":")
value   = Some("props, dangerous-props")
comment = None
term    = Some("\n")
```

</details>


## contributing

`oraide` is not yet ready for contributions.

## license ([AGPLv3])

Read [LICENSE-AGPLv3] for details.

[LICENSE-AGPLv3]: ./LICENSE-AGPLv3
[AGPLv3]: https://www.gnu.org/licenses/agpl-3.0
[libraries]: https://en.wikipedia.org/wiki/Library_(computing)
[OpenRA]: https://openra.net
[RTS]: https://en.wikipedia.org/wiki/Real-time_strategy
[game engine]: https://en.wikipedia.org/wiki/Game_engine
[lex]: https://en.wikipedia.org/wiki/Lexical_analysis
[MiniYaml]: https://www.openra.net/book/glossary.html#miniyaml
[linting]: https://en.wikipedia.org/wiki/Lint_%28software%29
[prop]: https://en.wikipedia.org/wiki/Theatrical_property
[actor]: https://www.openra.net/book/glossary.html#actor
[YAML]: https://en.wikipedia.org/wiki/YAML
[Rust]: https://www.rust-lang.org/
[rust-toolchain]: ./rust-toolchain
[Cargo]: https://doc.rust-lang.org/cargo/
[Visual Studio Code]: https://code.visualstudio.com/
