# `oraide-vscode`

[Visual Studio Code](https://code.visualstudio.com/) extension for [OpenRA](https://github.com/OpenRA/OpenRA) that aims to make modding easier, faster, and more intuitive.

---

## Installation

This extension is not yet available on [the VSCode Marketplace](https://marketplace.visualstudio.com/vscode) so you must use the following command to build and install from source.

```
$ yarn install-ext
```

<details>
<summary>What does that command do?</summary>

Read the `scripts.install-ext` part of [`package.json`](./package.json) to learn more!

Alternatively, assuming you have [`jq`](https://stedolan.github.io/jq/) installed, the following command will print out the commands encompassed in the above command.

```
$ cat package.json | jq -r '.scripts."install-ext"'
```
</details>

You should now be able to find an extension named _OpenRA IDE_ in your VSCode _extensions_ viewlet's _enabled_ section.

## Development

### Requirements

- [`node`](https://nodejs.org/) version `10.7`+
- [`yarn`](https://yarnpkg.com/en/docs/install) version `1.15`+
- the latest version of [Visual Studio Code](https://code.visualstudio.com/)

I suggest using [`nvm`](https://github.com/creationix/nvm) to manage your Node.js installation & versions.

### Building the extension

```
$ yarn build-ext
```

### Understanding the source code

Read the [./docs/dev/architecture.md](./docs/dev/architecture.md) file to get a high-level overview of the codebase.

<details>
<summary>What does that command do?</summary>

Read the `scripts.build-ext` part of [`package.json`](./package.json) to learn more!

Alternatively, assuming you have [`jq`](https://stedolan.github.io/jq/) installed, the following command will print out the commands encompassed in the above command.

```
$ cat package.json | jq -r '.scripts."build-ext"'
```
</details>

You should now have a `./out/oraide.vsix` file which you can [install with a custom `yarn` script](./README.md#installation) or [manually](https://code.visualstudio.com/docs/editor/extension-gallery#_install-from-a-vsix).

Read [./docs/dev/architecture.md](./docs/dev/architecture.md) for an overview of the extension's architecture.

### Testing (manual & autmated)

TODO