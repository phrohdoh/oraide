# notes, tips, etc. for developers

"developers" in the heading really means non-end-users.

## easier MiniYaml checking via a [named pipe]

MiniYaml files are [linted], at the time of writing, like so.

```shell
# from the root of this repo
cargo run -- check path/to/your/file.yaml
```

As shown, `check` takes a _file-path_, not MiniYaml text directly.

The result of this is that if you want to verify behavior, you must create a
file with the desired text then run `check` against that file.

That is a bit lame.

We can improve this with the help of [named pipe]s!

It is recommended to create 2 terminal instances (viewing them side-by-side),
which will:
- manage the pipe (creation, filling, deletion)
- run the `check` command over the pipe, in a loop

<small>in the 1st terminal:</small>
```shell
# create the pipe (repo root, or wherever)
# (feel free to use a different name)
mkfifo fifo-check-me
```

<small>in the 2nd terminal:</small>
```shell
# run check over the pipe, until interrupted (ctrl-c)
while :; do
    cargo run -- check fifo-check-me
done
```

Now you can fill the pipe with text which will in turn run the `check` command
over the given text as if it were in its own file, as many times as you want.

<small>in the 1st terminal:</small>
```shell
printf 'hello: world\n' > fifo-check-me
```

Use `rm` to delete the named pipe once you no longer want it.
Of course, you can recreate it whenever you need.

### bonus: send existing files through the pipe

<small>in the 1st terminal:</small>
```shell
cat path/to/existing/file.yaml > fifo-check-me
```

[named pipe]: https://en.wikipedia.org/wiki/Named_pipe
[linted]: https://en.wikipedia.org/wiki/Lint_(software)
