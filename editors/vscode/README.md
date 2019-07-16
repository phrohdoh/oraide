# `oraide-vscode`

[Visual Studio Code](https://code.visualstudio.com/) extension for
[OpenRA](https://github.com/OpenRA/OpenRA) that aims to make modding easier,
faster, and more intuitive.

---

## Installation

This extension is not yet available on [the VSCode
Marketplace](https://marketplace.visualstudio.com/vscode) so you must use the
following command to build and install from source.

```
$ yarn install-ext
```

<details>
<summary>What does that command do?</summary>

Read the `scripts.install-ext` part of [`package.json`](./package.json) to
learn more!

Alternatively, assuming you have [`jq`](https://stedolan.github.io/jq/)
installed, the following command will print out the commands encompassed in the
above command.

```
$ cat package.json | jq -r '.scripts."install-ext"'
```
</details>

You should now be able to find an extension named _OpenRA IDE_ in your VSCode
_extensions_ viewlet's _enabled_ section.

## Extensions to pair this one with

[indent-rainbow](https://marketplace.visualstudio.com/items?itemName=oderwat.indent-rainbow)
can help you spot those pesky instances of invalid indentation quicker

[omnisharp-vscode](https://marketplace.visualstudio.com/items?itemName=ms-vscode.csharp)
is the de facto extension for .NET / C# work (if you're writing custom traits)

If you think users would benefit from listing another extension here please
[open a ticket suggesting another extension]!

## Development

### Requirements

- [`node`](https://nodejs.org/) version `10.7`+
- [`yarn`](https://yarnpkg.com/en/docs/install) version `1.15`+
- the latest version of [Visual Studio Code](https://code.visualstudio.com/)

I suggest using [`nvm`](https://github.com/creationix/nvm) to manage your
Node.js installation & versions.

### Building the extension

```
$ yarn build-ext
```

<details>
<summary>What does that command do?</summary>

Read the `scripts.build-ext` part of [`package.json`](./package.json) to
learn more!

Alternatively, assuming you have [`jq`](https://stedolan.github.io/jq/)
installed, the following command will print out the commands encompassed in the
above command.

```
$ cat package.json | jq -r '.scripts."build-ext"'
```
</details>

You should now have a `./out/oraide.vsix` file which you can [install with a
custom `yarn` script](./README.md#installation) or
[manually](https://code.visualstudio.com/docs/editor/extension-gallery#_install-from-a-vsix).

### Understanding the source code

Read [./docs/dev/architecture.md](./docs/dev/architecture.md) for an overview of
the extension's architecture.

### Testing (manual & automated)

TODO

[open a ticket suggesting another extension]: https://github.com/Phrohdoh/oraide/issues/new?title=VSCode%20Extension%20Request:%20Link%20to%20extension%20`NAME-HERE`&body=\<!--%20please%20replace%20NAME-HERE%20in%20the%20title%20and%20add%20a%20link%20to%20the%20extension%20in%20the%20body%20--\>