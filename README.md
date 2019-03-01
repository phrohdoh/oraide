# `oraide`

A collection of tools with the aim of lowering the barrier to entry for -based game development.

## License

Proprietary.

## Project lead / point-of-contact

Phrohdoh (taryn@phrohdoh.com)

## Terms to know

**OpenRA ([link](https://openra.net))**: An open-source ([GPLv3+](https://www.gnu.org/licenses/quick-guide-gplv3.html)) game engine implemented in [C#](https://docs.microsoft.com/en-us/dotnet/csharp/) on top of the [.NET Framework](https://en.wikipedia.org/wiki/.NET_Framework)

**MiniYaml**: OpenRA's custom configuration language which hijacked the `.yaml` extension (note: it **is not** valid [YAML](https://yaml.org/spec/1.2/spec.html))

**SDK ([link](https://github.com/OpenRA/OpenRAModSDK/))**: The official template for OpenRA-based games that comes bundled with utilitiy scripts

## General project structure

The `oraide` project is comprised of multiple components (some of which have dependencies on others).

#### oraml

A  library (implemented in [Rust](https://www.rust-lang.org)) that converts raw text, such as files on-disk, to MiniYaml trees.

#### oraws (not yet implemented)

A library (implemented in Rust) that allows programmatic management of SDK-based projects.

Note that I am considering implementing a custom SDK-like tool (think of it like `cargo` for OpenRA-based games) at which point `oraws` will exist to manage those projects.