# `oraide`

A collection of tools with the aim of lowering the barrier to entry for -based game development.

## License

Proprietary.

## Project lead / point-of-contact

Taryn (a.k.a. Phrohdoh taryn@phrohdoh.com)

## Terms to know

**OpenRA ([link](https://openra.net))**: An open-source ([GPLv3+](https://www.gnu.org/licenses/quick-guide-gplv3.html)) game engine implemented in [C#](https://docs.microsoft.com/en-us/dotnet/csharp/) on top of the [.NET Framework](https://en.wikipedia.org/wiki/.NET_Framework)

**MiniYaml**: OpenRA's custom configuration language which hijacked the `.yaml` extension (note: it **is not** valid [YAML](https://yaml.org/spec/1.2/spec.html))

**SDK ([link](https://github.com/OpenRA/OpenRAModSDK/))**: The official template for OpenRA-based games that comes bundled with utilitiy scripts

## General project structure

The `oraide` project is comprised of multiple components (some of which have dependencies on others).

| component | description |
|-|-|
| [oraml](./src/libs/oraml/README.md) | A library that converts text to MiniYaml trees |
| [oraws](./src/libs/oraws/README.md) | A library that allows programmatic management of SDK-based projects |