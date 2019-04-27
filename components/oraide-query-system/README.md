# `oraide-query-system`

An on-demand, incremental computation system for the OpenRA IDE (`oraide`)
project built on top of [`salsa`].

---

This system allows us to compute _something_ once and [_memeoize_] the result,
which means the next time we compute _the same thing_ we can just clone the
results from the cache [_iff_] all of the computation's dependencies
remain unchanged.

Think about computing a tree for a given file path and look at the following
graph-like illustration.

> NOTE: Arrow direction indicates "has a dependency on" (`foo() <- bar()` means
> "`bar` has a dependency on `foo`")

```
text_of("infantry.yaml") <- tree_of("infantry.yaml") <-+
                                                       | <- type-checking
text_of("vehicles.yaml") <- tree_of("vehicles.yaml") <-+
```

Here we can see:

- `type-checking` depends on both
    - the result of `tree_of("infantry.yaml")`
        - which depends on the result of `text_of("infantry.yaml")`
    - the result of `tree_of("vehicles.yaml")`
        - which depends on the result of `text_of("vehicles.yaml")`

We only need to recompute `type-checking` when either of its
dependencies change.

A good question to ask then is whether we need to recompute `type-checking`
when the result of `text_of("infantry.yaml")` changes.

The answer is "[_iff_] the result of `tree_of("infantry.yaml")` changes."

In theory the tree may not change if, for example, only insignificant
whitespace was added to `infantry.yaml`.

In practice [`oraide-parser-miniyaml`]'s `Tree` considers all whitespace
significant, but the theory stands.

## How does this package implement this functionality?

This package exports a `Database` type which contains [_memeoized_]
computation results.

The [`oraide-parser-miniyaml`] package exports some types for file parsing:

- `ParserCtx`, defines inputs and queries for a `Database`
- `ParserCtxExt`, helper functions that make using `Database` for parsing
a bit easier

### Example

```rust
use oraide_query_system::Database;

use oraide_parser_miniyaml::{
    ParserCtx,
    ParserCtxExt,
    Tree,
};

// this is *your function*, not implemented in `oraide-parser-miniyaml`
let file_contents: String = get_contents_of("example.yaml");

let mut db = Database::default();

// the first argument is a string used to identify a file by humans,
// in most cases it will probably be the path to the file
//
// `db.add_file` is a helper function introduced by `ParserCtxExt`
// that sets inputs and queries the database
//
// you should read its implementation to better understand
let file_id = db.add_file("example.yaml", file_contents);

// `db.file_tree` is a *query* since you are *querying* the database for a tree
// given a file ID
let tree: Tree = db.file_tree(file_id);
```

Now if we were to call `db.file_tree(file_id)` again (with the same `file_id`
value) _without any relevant inputs having changed_ (such as the file's text)
we would _quickly_ return the same results that were computed the first time it
was invoked.

[`salsa`]: https://github.com/salsa-rs/salsa
[_memeoize_]: https://en.wikipedia.org/wiki/Memoization
[_memeoized_]: https://en.wikipedia.org/wiki/Memoization
[_iff_]: https://en.wikipedia.org/wiki/If_and_only_if
[`oraide-parser-miniyaml`]: ../oraide-parser-miniyaml/README.md