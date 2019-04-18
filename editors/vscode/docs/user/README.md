# _OpenRA IDE_ - Documentation for Users

## Server Configuration

The server can be configured in your [VSCode settings](https://code.visualstudio.com/docs/getstarted/settings).

Your `settings.json` may look like the following:

```json
{
    "oraide.server.shouldLogToFile": true,
    "oraide.server.exePath": "path/to/your-custom-executable",
    "oraide.trace.server": "verbose",
}
```

Configuration items have 2 parts:
- key (prefixed with `oraide` for the _OpenRA IDE_ extension)
- value

All of the server configuration keys have the `server` prefix with the exception of `oraide.trace.server` which is enforced by a third-party package that the extension uses.

Let's go over these in detail.

### `oraide.server.shouldLogToFile`

The value of this item:
- must be a boolean (either `true` or `false`)
- will determine whether the server writes logs to a file named `oraide-<timestamp>.log` in your VSCode workspace root

If you have experiencing issues with _OpenRA IDE_ please set this value to `true`, restart VSCode, perform some actions (such as hovering over one of an actor's traits in your MiniYaml), and read the newest log file created.

The _OpenRA IDE_ developers will ask you to do exactly this if you report an issue / ask for help so it'd be a good use of everyone's time to do it ahead-of-time.  We may even ask you to send us this file so it'll be helpful to have it on-hand.

### `oraide.server.exePath`

> NOTE: This configuration item is _entirely optional_, if this section doesn't make sense to you then you don't want to include this in your `settings.json` (unless an _OpenRA IDE_ developer is helping you troubleshoot).

> NOTE: This is typically only used by the _OpenRA IDE_ developers or "power users."

The value of this item:
- must be a string
- will override the built-in name/path of the language server executable

You might change this for numerous reasons, such as (but not limited to):
- general logging
- debugging the server by `tee`-ing input and output
- using `oraide_language_server`'s behaviour as a reference for your own language server implementation
- simply using a different server implementation (maybe you're writing an `oraide_language_server` competitor :O)

### `oraide.trace.server`

The value of this item:
- must be one of `messages`, `off`, or `verbose`
- will determine whether the client (the VSCode extenion) logs all messages to and from the server in the _Output_ viewlet (which you can read more about [here](https://code.visualstudio.com/docs/editor/debugging))