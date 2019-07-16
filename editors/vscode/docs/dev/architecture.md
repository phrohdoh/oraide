# _OpenRA IDE_ - Architecture

The `editors/vscode/` directory contains the Visual Studio Code extension
(sometimes referred to as `oraide-vscode`).

## Prerequisite Knowledge

This document assumes you have read and understood the [Language Server
Protocol](https://microsoft.github.io/language-server-protocol/overview) site
(including the [spec](https://microsoft.github.io/language-server-protocol/specification)).

You don't have to be an LSP expert, but a basic understanding of the principles
will be useful.

## Architecture

The [vscode-languageclient node
package](https://www.npmjs.com/package/vscode-languageclient), which is [open
source](https://github.com/Microsoft/vscode-languageserver-node), is used to
handle most of the heavy-lifting, such as sending messages to the server and
receiving responses.  Yay for code reuse!

### Management of the Server Process

We spawn the server as a child process by searching the user's `$PATH` (or
`%PATH%` for the Windows users, `$env:PATH` on PowerShell) for the `ora`
executable, unless the `oraide.server.exePath` configuration value is set, in
which case we use the value provided (which can either be an executable's name
or path).

This process is managed by the previously-mentioned `vscode-languageclient`
package.

If communication with the server process is lost (if you kill the server with
`kill -9 <server's pid>`, for example) the server will be re-launched and the
initialization 'handshake' will happen again.

### Activation of the Extension

The _OpenRA IDE_ extension is activated when a document with a so-called
_Language ID_ of `miniyaml` is opened.

By convention (and some hard-coding) OpenRA MiniYaml files have the `.yaml`
extension so, for ease of use, we register `.yaml`, `.miniyaml`, and `.oraml`
file types as having the _Language ID_ `miniyaml`.

This is done in 2 places:

- in [`extension.ts`](../../src/extension.ts) by setting
`clientOptions.documentSelector`, to register extension activation
- in [`package.json`](../../package.json) at `contributes.languages`, to
register the mentioned file types with the `miniyaml` _Language ID_

Some day we may want to only activate the extension if either a `mod.yaml` (a
hard-coded file name in OpenRA) or `mod.config` (a file in the root of the
[OpenRA SDK](https://github.com/OpenRA/OpenRAModSDK)) are present.

## Command Registration

We then register the commands that the extension makes available to the user in
our `registerCommands` function.

This is a simple process of calling `commands.registerCommand(id, fn)`.

```typescript
commands.registerCommand('oraide.server.restart', async () => {
    // code that restarts the server process
});
```

For each call to `commands.registerCommand` we need an associated object in the
`package.json`'s `commands` list.

```json
{
    "commands": [
        {
            "command": "oraide.server.restart",
            "title": "Restart the OpenRA Language Server",
            "description": "Sometimes, it's just best to try turning it off and on again",
            "category": "OpenRA"
        }
    ],
}
```

Here you can see that `oraide.server.restart` is used in both places, it is what
ties the function to the actual command the user can run.

When the user runs the _Restart the OpenRA Language Server_ command the function
(`async () => { .. }`) will be invoked.

---

## Continued Reading

Read the [user docs](../user/README.md) to learn about installing, configuring,
and using the extension.