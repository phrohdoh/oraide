# _OpenRA IDE_ - Documentation for Users

## Extension Installation

This section should be filled out once the extension is published to the
extension marketplace.

For the time being read [the extension's top-level
readme](../../README.md#installation)
for instructions on building from source and installing the extension.

## Server Configuration

The server can be configured in your [VSCode
settings](https://code.visualstudio.com/docs/getstarted/settings).

Your `settings.json` may look like the following:

```json
{
    "oraide.server.shouldLogToFile": true,
    "oraide.server.exePath": "path/to/your-custom-executable",
    "oraide.server.exeArgs": [ "your", "custom", "args" ],
    "oraide.trace.server": "verbose",
}
```

Configuration items have 2 parts:
- key (prefixed with `oraide` for the _OpenRA IDE_ extension)
- value

All of the server configuration keys have the `server` prefix with the exception
of `oraide.trace.server` which is enforced by a third-party package that the
extension uses.

Let's go over these in detail.

### `oraide.server.shouldLogToFile`

> NOTE: This configuration item is _entirely optional_, if this section doesn't
make sense to you then you don't want to include this in your `settings.json`
(unless an _OpenRA IDE_ developer is helping you troubleshoot).

The value of this item:
- must be a boolean (either `true` or `false`)
- will determine whether the server writes logs to a file named
`<local timestamp of server start>.log` in `<workspace root>/.oraide/logs/`

If you experience issues with _OpenRA IDE_ please set this value to
`true`, restart VSCode, perform some actions (such as hovering over one of an
actor's traits in your MiniYaml), and read the newest log file created.

The _OpenRA IDE_ developers will ask you to do exactly this if you report an
issue / ask for help so it'd be a good use of everyone's time to do it
ahead-of-time.  We may even ask you to send us this file so it'll be helpful to
have it on-hand.

### `oraide.server.exePath`

> NOTE: This configuration item is _entirely optional_, if this section doesn't
make sense to you then you don't want to include this in your `settings.json`
(unless an _OpenRA IDE_ developer is helping you troubleshoot).

> NOTE: This is typically only used by the _OpenRA IDE_ developers or
"power users."

The value of this item:
- must be a string
- will override the built-in name/path of the language server executable

You might change this for numerous reasons, such as (but not limited to):
- general logging
- debugging the server by `tee`-ing input and output
- using `ora`'s behavior as a reference for your own language server
implementation
- simply using a different server implementation (maybe you're writing an `ora`
competitor :O)

### `oraide.server.exeArgs`

> NOTE: This configuration item is _entirely optional_, if this section doesn't
make sense to you then you don't want to include this in your `settings.json`
(unless an _OpenRA IDE_ developer is helping you troubleshoot).

> NOTE: This is typically only used by the _OpenRA IDE_ developers or
"power users."

The value of this item:
- must be an array of strings
- will override the built-in arguments passed to the `ora` (or
`oraide.server.exePath`) executable

To understand how to use this configuration value you need to know how the
language server process is managed by this VSCode extension.

The first step is finding an executable file to launch.  If
`oraide.server.exePath` is specified in configuration then the file at that path
is used.  Otherwise your `$PATH` is searched for an executable file named `ora`.

Once an executable is located it is spawned with an argument list.  The default
argument list is `[ "ide" ]` which launches the `ora` tool in [IDE] mode.

Let's pretend for a moment that this IDE mode can be made bug-free with the
command-line argument `--no-bugs-pls`.

To enable this you'd set `oraide.server.exeArgs` to `[ "ide", "--no-bugs-pls" ]`
which would look like the following in your settings JSON.

```json
{
    "oraide.server.exeArgs": [ "ide", "--no-bugs-pls" ]
}
```

This is essentially the same as running the following in your shell.

```shell
$ ora ide --no-bugs-pls
```

Which is actually what this extension does!

---

Note that since you can run a custom executable as the server process there is
no guarantee that arguments for one language server implementation will work
for another implementation.

If you aren't setting `oraide.server.exePath` you can essentially ignore the
previous statement and pretend you never read it.

### `oraide.trace.server`

> NOTE: This configuration item is _entirely optional_, if this section doesn't
make sense to you then you don't want to include this in your `settings.json`
(unless an _OpenRA IDE_ developer is helping you troubleshoot).

The value of this item:
- must be one of `messages`, `off`, or `verbose`
- will determine whether the client (the VSCode extension) logs all messages to
and from the server in the _Output_ viewlet (which you can read more about
[here](https://code.visualstudio.com/docs/editor/debugging))

[IDE]: https://en.wikipedia.org/wiki/Integrated_development_environment